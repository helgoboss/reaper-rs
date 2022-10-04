#![doc(html_root_url = "https://docs.rs/reaper-macros/0.1.0")]
#![allow(renamed_and_removed_lints)]
#![deny(broken_intra_doc_links)]

//! This crate is part of [reaper-rs](https://github.com/helgoboss/reaper-rs) and contains a
//! [simple attribute macro](attr.reaper_extension_plugin.html) to simplify bootstrapping REAPER
//! extension plug-ins.
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

/// Macro for easily bootstrapping a REAPER extension plug-in.
///
/// Use the macro like this:
/// ```no_run
/// use std::error::Error;
/// use reaper_macros::reaper_extension_plugin;
/// use reaper_low::PluginContext;
/// use reaper_medium::ReaperSession;
///
/// #[reaper_extension_plugin]
/// fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
///     let session = ReaperSession::load(context);
///     session.reaper().show_console_msg("Hello world from reaper-rs medium-level API!");
///     Ok(())
/// }
/// ```
///
/// If you want to start with a preconfigured high-level `Reaper` instance right away, use the macro
/// like this (please note that the high-level API has not been published yet):
///
/// ```no_run,ignore
/// use std::error::Error;
/// use reaper_macros::reaper_extension_plugin;
/// use reaper_high::Reaper;
///
/// #[reaper_extension_plugin(
///     name = "Example",
///     support_email_address = "support@example.org"
/// )]
/// fn plugin_main() -> Result<(), Box<dyn Error>> {
///     Reaper::get().show_console_msg("Hello world from reaper-rs high-level API!");
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn reaper_extension_plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse attributes
    let args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let args = match ReaperExtensionPluginMacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };
    // Parse function which is annotated with that attribute
    let main_function = syn::parse_macro_input!(input as syn::ItemFn);
    // Check if it's a low-level or high-level plug-in.
    // If the function has one parameter, it's a low-level plug-in, otherwise a high-level one.
    match main_function.sig.inputs.len() {
        0 => {
            // No function parameter. Must be a high-level plug-in.
            generate_high_level_plugin_code(args, main_function)
        }
        1 => {
            // One function parameter. Must be a low-level plug-in.
            generate_low_level_plugin_code(main_function)
        }
        _ => panic!("REAPER extension plugin function must have "),
    }
}

fn generate_low_level_plugin_code(main_function: syn::ItemFn) -> TokenStream {
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        /// Windows entry and exit point for clean-up.
        ///
        /// Called by REAPER for Windows once at startup time with DLL_PROCESS_ATTACH and once
        /// at exit time or manual unload time (if plug-in initialization failed) with
        /// DLL_PROCESS_DETACH.
        #[cfg(target_family = "windows")]
        #[allow(non_snake_case)]
        #[no_mangle]
        extern "system" fn DllMain(
            hinstance: reaper_low::raw::HINSTANCE,
            reason: u32,
            _: *const u8,
        ) -> u32 {
            if (reason == reaper_low::raw::DLL_PROCESS_DETACH) {
                unsafe {
                    reaper_low::execute_plugin_destroy_hooks();
                }
            }
            1
        }

        /// Linux entry and exit point for getting hold of the SWELL function provider.
        ///
        /// See `reaper_vst_plugin!` macro why clean-up is neither necessary nor desired  on Linux
        /// at the moment.
        ///
        /// Called by REAPER for Linux once at startup time with DLL_PROCESS_ATTACH and once
        /// at exit time or manual unload time (if plug-in initialization failed) with
        /// DLL_PROCESS_DETACH.
        ///
        /// In case anybody wonders where's the SWELL entry point for macOS:
        /// `swell-modstub-custom.mm`.
        #[cfg(target_os = "linux")]
        #[allow(non_snake_case)]
        #[no_mangle]
        extern "C" fn SWELL_dllMain(
            hinstance: reaper_low::raw::HINSTANCE,
            reason: u32,
            get_func: Option<
                unsafe extern "C" fn(
                    name: *const std::os::raw::c_char,
                ) -> *mut std::os::raw::c_void,
            >,
        ) -> std::os::raw::c_int {
            if (reason == reaper_low::raw::DLL_PROCESS_ATTACH) {
                reaper_low::register_swell_function_provider(get_func);
            }
            1
        }

        /// Entry point for the REAPER extension plug-in.
        ///
        /// This is called by REAPER at startup time.
        #[no_mangle]
        unsafe extern "C" fn ReaperPluginEntry(h_instance: ::reaper_low::raw::HINSTANCE, rec: *mut ::reaper_low::raw::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            let static_context = reaper_low::static_extension_plugin_context();
            ::reaper_low::bootstrap_extension_plugin(h_instance, rec, static_context, #main_function_name)
        }

        #main_function
    };
    tokens.into()
}

fn generate_high_level_plugin_code(
    args: ReaperExtensionPluginMacroArgs,
    main_function: syn::ItemFn,
) -> TokenStream {
    let plugin_name = args
        .name
        .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());
    let plugin_version = env!("CARGO_PKG_VERSION").to_string();
    let support_email_address = args
        .support_email_address
        .expect("support_email_address missing");
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[::reaper_macros::reaper_extension_plugin]
        fn low_level_plugin_main(context: ::reaper_low::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
            let crash_info = ::reaper_high::CrashInfo {
                plugin_name: #plugin_name.to_string(),
                plugin_version: #plugin_version.to_string(),
                support_email_address: #support_email_address.to_string(),
            };
            ::reaper_high::Reaper::setup_with_defaults(context, ::reaper_high::create_terminal_logger(), crash_info);
            #main_function_name()
        }

        #main_function
    };
    tokens.into()
}

/// Arguments passed to the [`reaper_extension_plugin`] macro.
///
/// [`reaper_extension_plugin`]: macro.reaper_extension_plugin.html
#[derive(Default, Debug, FromMeta)]
#[darling(default)]
struct ReaperExtensionPluginMacroArgs {
    /// Plug-in name which will appear in error reports.
    ///
    /// Only used for high-level plug-ins. Optional, defaults to package name.
    name: Option<String>,
    /// Support e-mail address which will appear in error reports.
    ///
    /// Necessary for high-level plug-in.
    support_email_address: Option<String>,
}

#[cfg(doctest)]
doc_comment::doctest!("../../../README.md");
