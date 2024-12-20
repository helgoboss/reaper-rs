use crate::Reaper;
use backtrace::Backtrace;
use reaper_low::Swell;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::os::raw::c_char;
use std::panic::PanicInfo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Handles crashes when they occur.
pub struct CrashHandler {
    config: CrashHandlerConfig,
}

/// Configuration of the crash handler.
pub struct CrashHandlerConfig {
    /// General information about the crashing REAPER plug-in.
    pub plugin_info: PluginInfo,
    /// What to log to the REAPER console, if enabled by user.
    pub crash_formatter: Box<dyn CrashFormatter>,
    /// Whether to log to the REAPER console (user can toggle this at runtime).
    pub console_logging_enabled: Arc<AtomicBool>,
    /// Whether to report to Sentry (user can toggle this at runtime).
    pub sentry_enabled: Arc<AtomicBool>,
}

/// Information about the plug-in, to be shown in crash logs.
#[derive(Clone, Debug)]
pub struct PluginInfo {
    /// Name of the plug-in.
    pub plugin_name: String,
    /// Just the version number, for example, "2.15.0".
    pub plugin_version: String,
    /// Longer version info, maybe including the commit hash.
    pub plugin_version_long: String,
    /// Email address to which the user should send crash-related info.
    pub support_email_address: String,
    /// URL which is presented to the user with the request to try with the latest version before reporting the error.
    pub update_url: String,
}

/// All available information about a particular crash.
pub struct CrashInfo<'a> {
    pub plugin_info: &'a PluginInfo,
    pub panic_info: &'a PanicInfo<'a>,
    pub backtrace: Option<&'a Backtrace>,
    pub console_enabled: bool,
    pub sentry_enabled: bool,
    pub sentry_error_id: Option<String>,
}

impl CrashHandler {
    /// Creates a new crash handler with the given configuration.
    pub fn new(config: CrashHandlerConfig) -> Self {
        Self { config }
    }

    /// Handles a particular crash, initiated by a panic.
    ///
    /// This must be called from the panic hook.
    pub fn handle_crash(&self, panic_info: &PanicInfo) {
        let console_enabled = self.config.console_logging_enabled.load(Ordering::Relaxed);
        let sentry_enabled = self.config.sentry_enabled.load(Ordering::Relaxed);
        if !console_enabled && !sentry_enabled {
            // Neither console logging nor Sentry logging is enabled. Special handling.
            // Log at least to stdout
            log_panic(panic_info, None);
            // Don't capture backtrace => fast!
            let crash_info = CrashInfo {
                plugin_info: &self.config.plugin_info,
                panic_info,
                backtrace: None,
                console_enabled: false,
                sentry_enabled: false,
                sentry_error_id: None,
            };
            // Don't open console => non-disruptive!
            let msg = self.config.crash_formatter.format(&crash_info);
            Reaper::get().show_console_msg_thread_safe(format!("!SHOWERR:{msg}"));
            return;
        }
        // At least one of console logging or Sentry is enabled
        // Capture backtrace => slow!
        let backtrace = Backtrace::new();
        // In any case, log backtrace to stdout (useful for devs and power users)
        log_panic(panic_info, Some(&backtrace));
        // If enabled, report to Sentry
        let sentry_error_id = if sentry_enabled {
            #[cfg(feature = "sentry")]
            {
                self.report_to_sentry(panic_info, &backtrace).ok()
            }
            #[cfg(not(feature = "sentry"))]
            {
                None
            }
        } else {
            None
        };
        // If enabled, log to REAPER console
        let crash_info = CrashInfo {
            plugin_info: &self.config.plugin_info,
            panic_info,
            backtrace: Some(&backtrace),
            console_enabled,
            sentry_enabled,
            sentry_error_id,
        };
        // Open console => disruptive!
        let msg = self.config.crash_formatter.format(&crash_info);
        let msg = if crash_info.console_enabled || crash_info.sentry_error_id.is_none() {
            // Into the face!
            msg
        } else {
            // Don't open console window
            format!("!SHOWERR:{msg}")
        };
        Reaper::get().show_console_msg_thread_safe(msg);
    }
}

pub trait CrashFormatter: 'static + Sync + Send {
    /// Creates the text that should be displayed to the user.
    fn format(&self, crash_info: &CrashInfo) -> String;
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

pub struct CrashEnvironment {}

pub struct DefaultConsoleMessageFormatter;

impl CrashFormatter for DefaultConsoleMessageFormatter {
    fn format(&self, crash_info: &CrashInfo) -> String {
        let module_info = ModuleInfo::capture();
        let (module_path, module_base_address_label, module_size_label) = match module_info {
            Err(_) => (hyphen(), hyphen(), hyphen()),
            Ok(mi) => (mi.path.clone(), mi.format_base_address(), mi.format_size()),
        };
        let reaper_version = Reaper::get().version();
        let update_url = &crash_info.plugin_info.update_url;
        let plugin_name = &crash_info.plugin_info.plugin_name;
        let plugin_version_long = &crash_info.plugin_info.plugin_version_long;
        let email_address = &crash_info.plugin_info.support_email_address;
        let panic_message = extract_panic_message(crash_info.panic_info);
        let intro = format!("
===== ATTENTION =====

Sorry, an unexpected error occurred in REAPER plug-in {plugin_name}. REAPER should continue to work but {plugin_name} might show unexpected behavior until restarting REAPER. If you feel like saving your project file at this point, better save it as a new file because this error could have messed up the plug-in state.

Are you running the latest version of {plugin_name}? Please check for updates at \"{update_url}\". If an update is available, please install it and try again.

If this happens even with the latest version, please report this error:

1. Prepare an e-mail containing:
  - The error information further below (IMPORTANT)
  - Some instructions on how to reproduce the error (IMPORTANT)

2. If possible, attach the following files:
  - Your REAPER project file (.rpp)
  - Your REAPER configuration file (reaper.ini)

3. Send it to {email_address}

Thank you for your support!
"
        );
        let cut_intro = format!(
            "
--- cut ---
Message: {panic_message}

REAPER version:      {reaper_version}
Module name:         {plugin_name}
Module version:      {plugin_version_long}
Module path:         {module_path}
Module base address: {module_base_address_label}
Module size:         {module_size_label}
"
        );
        let cut_outro = "
--- cut ---

"
        .to_string();

        let backtrace = FormattedBacktrace(crash_info.backtrace);
        let components = if crash_info.sentry_enabled {
            // Sentry is enabled
            if let Some(error_id) = &crash_info.sentry_error_id {
                // Error has been reported to Sentry successfully
                &[intro, cut_intro, format!("Error ID: {error_id}"), cut_outro]
            } else {
                // Reporting to Sentry failed
                &[
                    intro,
                    cut_intro,
                    format!("Automatic error reporting failed!\n\nBacktrace: {backtrace}"),
                    cut_outro,
                ]
            }
        } else {
            // Sentry is disabled
            &[
                intro,
                cut_intro,
                format!("Backtrace: {backtrace}"),
                cut_outro,
            ]
        };
        components.join("\n").to_string()
    }
}

struct FormattedBacktrace<'a>(Option<&'a Backtrace>);

impl<'a> Display for FormattedBacktrace<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(backtrace) = self.0 {
            write!(f, "\n\n{backtrace:#?}")?;
        } else {
            f.write_str("-")?;
        }
        Ok(())
    }
}

pub fn log_panic(panic_info: &PanicInfo, backtrace: Option<&Backtrace>) {
    tracing::error!(
        message = extract_panic_message(panic_info),
        backtrace = format!("{backtrace:#?}")
    );
}

#[derive(Default)]
pub(crate) struct ModuleInfo {
    pub base_address: usize,
    pub size: Option<usize>,
    pub path: String,
}

impl ModuleInfo {
    pub fn capture() -> Result<ModuleInfo, &'static str> {
        let hinstance = Reaper::get()
            .medium_reaper()
            .plugin_context()
            .h_instance()
            .ok_or("couldn't obtain HINSTANCE/HMODULE")?;
        #[cfg(target_family = "windows")]
        {
            let info = ModuleInfo {
                base_address: hinstance.as_ptr() as usize,
                size: determine_module_size(hinstance),
                path: determine_module_path(hinstance),
            };
            Ok(info)
        }
        #[cfg(not(target_family = "windows"))]
        {
            let info = ModuleInfo {
                base_address: 0,
                size: None,
                path: determine_module_path(hinstance),
            };
            Ok(info)
        }
    }

    pub fn format_base_address(&self) -> String {
        format_as_hex(self.base_address)
    }

    pub fn format_size(&self) -> String {
        self.size.map(format_as_hex).unwrap_or_else(hyphen)
    }
}

fn determine_module_path(hinstance: reaper_medium::Hinstance) -> String {
    if !Swell::is_available_globally() {
        return String::new();
    }
    let (cstring, size) = with_string_buffer(1000, |buf, max_size| unsafe {
        Swell::get().GetModuleFileName(hinstance.as_ptr(), buf, max_size as _)
    });
    if size == 0 {
        return String::new();
    }
    cstring.to_string_lossy().to_string()
}

fn with_string_buffer<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (CString, T) {
    let vec: Vec<u8> = vec![1; max_size as usize];
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    let result = fill_buffer(raw, max_size as i32);
    let string = unsafe { CString::from_raw(raw) };
    (string, result)
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
    format!("0x{number:x}")
}

fn hyphen() -> String {
    "-".to_string()
}

#[cfg(feature = "sentry")]
mod sentry_impl {
    use super::*;
    use sentry::integrations::backtrace::backtrace_to_stacktrace;
    use sentry::integrations::panic::message_from_panic_info;
    use sentry::protocol::{Event, Exception, Mechanism};
    use sentry::{Hub, Level};

    impl CrashHandler {
        /// Returns the error ID.
        pub(crate) fn report_to_sentry(
            &self,
            panic_info: &PanicInfo,
            backtrace: &Backtrace,
        ) -> Result<String, &'static str> {
            // This is inspired by sentry-panic-0.35.0 function "event_from_panic_info".
            // We don't use the original because it captures a backtrace. But we already
            // have one!
            let msg = message_from_panic_info(panic_info);
            let exception = Exception {
                ty: "panic".into(),
                mechanism: Some(Mechanism {
                    ty: "panic".into(),
                    handled: Some(false),
                    ..Default::default()
                }),
                value: Some(msg.to_string()),
                stacktrace: backtrace_to_stacktrace(backtrace),
                ..Default::default()
            };
            let mut extra = sentry::types::protocol::v7::Map::new();
            extra.insert(
                "reaper_version".to_string(),
                Reaper::get().version().to_string().into(),
            );
            if let Ok(info) = ModuleInfo::capture() {
                extra.insert("module_path".to_string(), info.path.clone().into());
                extra.insert(
                    "module_base_address".to_string(),
                    info.format_base_address().into(),
                );
                extra.insert("module_size".to_string(), info.format_size().into());
            }
            let event = Event {
                exception: vec![exception].into(),
                level: Level::Fatal,
                extra,
                ..Default::default()
            };
            // This is inspired by sentry-panic-0.35.0 function "panic_handler"
            let hub = Hub::current();
            let Some(client) = hub.client() else {
                return Err("no sentry client bound");
            };
            let uuid = hub.capture_event(event);
            if uuid.is_nil() {
                return Err("capturing sentry event didn't work");
            }
            client.flush(None);
            Ok(uuid.to_string())
        }
    }
}
