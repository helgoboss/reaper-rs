use std::ffi::CString;
use std::os::raw::c_char;
use std::panic::PanicInfo;

use crate::Reaper;
use backtrace::Backtrace;
use reaper_low::Swell;

/// Creates a panic hook which logs the error both to the logging system and optionally to REAPER
/// console. This is just a convenience function. You can easily write your own panic hook if you
/// need further customization. Have a look at the existing implementation and used helper
/// functions.
pub fn create_reaper_panic_hook(
    console_msg_formatter: Option<
        impl Fn(&PanicInfo, &Backtrace) -> String + 'static + Sync + Send,
    >,
) -> Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send> {
    Box::new(move |panic_info| {
        let backtrace = Backtrace::new();
        log_panic(panic_info, &backtrace);
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
    pub update_url: String,
}

pub fn create_default_console_msg_formatter(
    crash_info: CrashInfo,
) -> impl Fn(&PanicInfo, &Backtrace) -> String {
    move |panic_info, backtrace| {
        let module_info = determine_module_info();
        let (module_path, module_base_address_label, module_size_label) = match module_info {
            Err(_) => (hyphen(), hyphen(), hyphen()),
            Ok(mi) => (
                mi.path,
                format_as_hex(mi.base_address),
                mi.size.map(format_as_hex).unwrap_or_else(hyphen),
            ),
        };
        let reaper_version = Reaper::get().version();
        // In the future, we might want to use [format!("!SHOWERR:] in order to not pop up console immediately.
        // But right now, it would hide errors: https://github.com/helgoboss/helgobox/issues/1304
        format!("

===== ATTENTION =====

Sorry, an unknown error occurred in REAPER plug-in {plugin_name}. REAPER should continue to work but {plugin_name} might show unexpected behavior until restarting REAPER. If you feel like saving your project file at this point, better save it as a new file because this error could have messed up the plug-in state. 

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

--- cut ---
REAPER version:      {reaper_version}
Module name:         {plugin_name}
Module version:      {plugin_version}
Module path:         {module_path}
Module base address: {module_base_address_label}
Module size:         {module_size_label}

Message: {panic_message}

{backtrace:#?}\
--- cut ---

",
                reaper_version = reaper_version,
                update_url = crash_info.update_url,
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

pub fn log_panic(panic_info: &PanicInfo, backtrace: &Backtrace) {
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

pub(crate) fn determine_module_info() -> Result<ModuleInfo, &'static str> {
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
