use slog::{error, o, Drain};
use std::backtrace::Backtrace;

use crate::{Reaper, ReaperSession};

use std::ffi::CString;
use std::io;
use std::panic::PanicInfo;

pub fn create_std_logger() -> slog::Logger {
    slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())
}

pub fn create_reaper_console_logger() -> slog::Logger {
    let sink = io::LineWriter::new(ReaperConsoleSink::new());
    let plain = slog_term::PlainSyncDecorator::new(sink);
    let drain = slog_term::FullFormat::new(plain).build().fuse();
    slog::Logger::root(drain, o!())
}

// TODO-low Async logging: https://github.com/gabime/spdlog/wiki/6.-Asynchronous-logging
// TODO-low Log to file in user home instead of to console
pub fn create_terminal_logger() -> slog::Logger {
    let sink = io::stdout();
    let plain = slog_term::PlainSyncDecorator::new(sink);
    let drain = slog_term::FullFormat::new(plain).build().fuse();
    slog::Logger::root(drain, o!())
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
        let backtrace = Backtrace::force_capture();
        log_panic(&logger, panic_info, &backtrace);
        if let Some(formatter) = &console_msg_formatter {
            let msg = formatter(panic_info, &backtrace);
            if let Ok(c_msg) = CString::new(msg) {
                ReaperSession::get().do_in_main_thread_asap(move || {
                    Reaper::get().medium().show_console_msg(c_msg.as_ref());
                });
            }
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

pub fn create_default_console_msg_formatter(
    email_address: &'static str,
) -> impl Fn(&PanicInfo, &Backtrace) -> String {
    move |panic_info, backtrace| {
        format!("

Sorry, an error occurred in a REAPER plug-in. It seems that a crash has been prevented. Better save your project at this point, preferably as a new file. It's recommended to restart REAPER before using the plug-in again. 

Please report this error:

1. Copy the following error information.
2. Paste the error information into an email and send it via email to {email_address}, along with the RPP file, your REAPER.ini file and some instructions how to reproduce the issue.

Thank you for your support!

--- cut ---
Message: {panic_message}

{backtrace}\
--- cut ---

",
                backtrace = backtrace,
                email_address = email_address,
                panic_message = extract_panic_message(panic_info)
        )
    }
}

pub fn log_panic(logger: &slog::Logger, panic_info: &PanicInfo, backtrace: &Backtrace) {
    error!(logger, "Plugin panicked";
        "message" => extract_panic_message(panic_info),
        "backtrace" => format!("{:?}", backtrace)
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
        let c_string = CString::new(buf)?;
        // TODO-medium If the panic happens in audio thread, this won't work!
        Reaper::get().medium().show_console_msg(c_string.as_ref());
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}
