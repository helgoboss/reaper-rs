use std::io;
use std::io::{LineWriter, Stdout};
use std::panic::PanicInfo;

use backtrace::Backtrace;
use slog::{error, o, Drain, Fuse};
use slog_term::{FullFormat, PlainSyncDecorator};

use crate::Reaper;

pub fn create_std_logger() -> slog::Logger {
    slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())
}

pub fn create_reaper_console_logger() -> slog::Logger {
    slog::Logger::root(create_reaper_console_drain(), o!())
}

// TODO-low Async logging: https://github.com/gabime/spdlog/wiki/6.-Asynchronous-logging
// TODO-low Log to file in user home instead of or in addition to console (for the latter we just
//  need to create a struct that contains both drains and implements the Drain trait by delegating
//  to both).
pub fn create_terminal_logger() -> slog::Logger {
    slog::Logger::root(create_stdout_drain(), o!())
}

fn create_stdout_drain() -> Fuse<FullFormat<PlainSyncDecorator<Stdout>>> {
    let sink = io::stdout();
    let plain = slog_term::PlainSyncDecorator::new(sink);
    slog_term::FullFormat::new(plain).build().fuse()
}

fn create_reaper_console_drain()
-> Fuse<FullFormat<PlainSyncDecorator<LineWriter<ReaperConsoleSink>>>> {
    let sink = io::LineWriter::new(ReaperConsoleSink::new());
    let plain = slog_term::PlainSyncDecorator::new(sink);
    slog_term::FullFormat::new(plain).build().fuse()
}

/// Creates a panic hook which logs the error both to the logging system and optionally to REAPER
/// console. This is just a convenience function. You can easily write your own panic hook if you
/// need further customization. Have a look at the existing implementation and used helper
/// functions.
pub fn create_reaper_panic_hook(
    logger: slog::Logger,
    console_msg_formatter: Option<
        impl Fn(&PanicInfo, &Backtrace) -> String + 'static + Sync + Send,
    >,
) -> Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send> {
    Box::new(move |panic_info| {
        let backtrace = Backtrace::new();
        log_panic(&logger, panic_info, &backtrace);
        if let Some(formatter) = &console_msg_formatter {
            let msg = formatter(panic_info, &backtrace);
            Reaper::get().show_console_msg_thread_safe(msg);
        }
    })
}

pub fn extract_panic_message(panic_info: &PanicInfo) -> String {
    let payload = panic_info.payload();
    match payload.downcast_ref::<&str>() {
        Some(p) => (*p).to_string(),
        None => match payload.downcast_ref::<String>() {
            Some(p) => p.clone(),
            None => String::from("Unknown error"),
        },
    }
}

pub struct CrashInfo {
    pub plugin_name: String,
    pub plugin_version: String,
    pub support_email_address: String,
}

pub fn create_default_console_msg_formatter(
    crash_info: CrashInfo,
) -> impl Fn(&PanicInfo, &Backtrace) -> String {
    move |panic_info, backtrace| {
        let module_info = determine_module_info();
        let (module_base_address_label, module_size_label) = match module_info {
            None => (hyphen(), hyphen()),
            Some(mi) => (
                format_as_hex(mi.base_address),
                mi.size.map(format_as_hex).unwrap_or_else(hyphen),
            ),
        };
        let reaper_version = Reaper::get().version();
        format!("

===== ATTENTION =====

Sorry, an unknown error occurred in REAPER plug-in {plugin_name}. REAPER should continue to work but {plugin_name} might show unexpected behavior until restarting REAPER. If you feel like saving your project file at this point, better save it as a new file because this error could have messed up the plug-in state. 

In any case, please report this error:

1. Prepare an e-mail containing:
    - The error information further below (IMPORTANT)
    - Some instructions on how to reproduce the error (IMPORTANT)
2. If possible, attach the following files: 
    - Your REAPER project file (.rpp)
    - Your REAPER configuration file (reaper.ini)
3. Send it to {email_address}

Thank you for your support!

--- cut ---
REAPER version:      {reaper_version}
Module name:         {plugin_name}
Module version:      {plugin_version}
Module base address: {module_base_address_label}
Module size:         {module_size_label}

Message: {panic_message}

{backtrace:#?}\
--- cut ---

",
                reaper_version = reaper_version,
                plugin_name = crash_info.plugin_name,
                plugin_version = crash_info.plugin_version,
                module_base_address_label = module_base_address_label,
                module_size_label = module_size_label,
                backtrace = backtrace,
                email_address = crash_info.support_email_address,
                panic_message = extract_panic_message(panic_info)
        )
    }
}

pub fn log_panic(logger: &slog::Logger, panic_info: &PanicInfo, backtrace: &Backtrace) {
    error!(logger, "Plugin panicked";
        "message" => extract_panic_message(panic_info),
        "backtrace" => format!("{:#?}", backtrace)
    );
}

struct ReaperConsoleSink {}

impl ReaperConsoleSink {
    fn new() -> ReaperConsoleSink {
        ReaperConsoleSink {}
    }
}

impl std::io::Write for ReaperConsoleSink {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let str_slice =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        // TODO-medium If the panic happens in audio thread, this won't work!
        Reaper::get().medium_reaper().show_console_msg(str_slice);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub(crate) struct ModuleInfo {
    pub base_address: usize,
    pub size: Option<usize>,
}

pub(crate) fn determine_module_info() -> Option<ModuleInfo> {
    #[cfg(target_family = "windows")]
    {
        let hinstance = Reaper::get()
            .medium_reaper()
            .plugin_context()
            .h_instance()?;
        let info = ModuleInfo {
            base_address: hinstance.as_ptr() as usize,
            size: determine_module_size(hinstance),
        };
        Some(info)
    }
    #[cfg(not(target_family = "windows"))]
    {
        None
    }
}

#[cfg(target_family = "windows")]
fn determine_module_size(hinstance: reaper_medium::Hinstance) -> Option<usize> {
    let size = unsafe {
        use winapi::um::processthreadsapi;
        use winapi::um::psapi;
        let process = processthreadsapi::GetCurrentProcess();
        if process.is_null() {
            return None;
        }
        use std::ptr::null_mut;
        let mut mi = psapi::MODULEINFO {
            lpBaseOfDll: null_mut(),
            SizeOfImage: 0,
            EntryPoint: null_mut(),
        };
        let success = psapi::GetModuleInformation(
            process,
            hinstance.as_ptr() as _,
            &mut mi as *mut _ as _,
            std::mem::size_of::<psapi::MODULEINFO>() as _,
        );
        if success == 0 {
            return None;
        }
        mi.SizeOfImage as _
    };
    Some(size)
}

fn format_as_hex(number: usize) -> String {
    format!("0x{:x}", number)
}

fn hyphen() -> String {
    "-".to_string()
}
