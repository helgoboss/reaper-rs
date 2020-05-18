#![doc(html_root_url = "https://docs.rs/reaper-macros/0.1.0")]

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
/// use reaper_low::ReaperPluginContext;
/// use reaper_medium::ReaperSession;
///
/// #[reaper_extension_plugin]
/// fn plugin_main(context: ReaperPluginContext) -> Result<(), Box<dyn Error>> {
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
/// use reaper_high::ReaperSession;
///
/// #[reaper_extension_plugin(email_address = "support@example.org")]
/// fn plugin_main() -> Result<(), Box<dyn Error>> {
///     let session = ReaperSession::get();
///     session.show_console_msg("Hello world from reaper-rs high-level API!");
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

/// No-op macro as work-around for https://github.com/magnet/metered-rs/issues/23.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn measure(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

fn generate_low_level_plugin_code(main_function: syn::ItemFn) -> TokenStream {
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        mod reaper_extension_plugin {
            /// Exposes the (hopefully) obtained handles.
            pub fn static_context() -> reaper_low::StaticReaperExtensionPluginContext {
                reaper_low::StaticReaperExtensionPluginContext {
                    get_swell_func: unsafe { GET_SWELL_FUNC },
                }
            }

            // Entry point for getting hold of the SWELL function provider.
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
                    INIT_GET_SWELL_FUNC.call_once(|| {
                        unsafe { GET_SWELL_FUNC = get_func };
                    });
                }
                // Give the C++ side of the plug-in the chance to initialize its SWELL function
                // pointers as well.
                #[cfg(not(target_os = "windows"))]
                unsafe { SWELL_dllMain_called_from_rust(hinstance, reason, get_func); }
                1
            }
            #[cfg(not(target_os = "windows"))]
            extern "C" {
                pub fn SWELL_dllMain_called_from_rust(
                   hinstance: reaper_low::raw::HINSTANCE,
                   reason: u32,
                   get_func: Option<
                       unsafe extern "C" fn(
                           name: *const std::os::raw::c_char,
                       ) -> *mut std::os::raw::c_void,
                   >,
                ) -> std::os::raw::c_int;
            }
            static mut GET_SWELL_FUNC: Option<
                unsafe extern "C" fn(
                    name: *const std::os::raw::c_char,
                ) -> *mut std::os::raw::c_void,
            > = None;
            static INIT_GET_SWELL_FUNC: std::sync::Once = std::sync::Once::new();
        }

        #[no_mangle]
        unsafe extern "C" fn ReaperPluginEntry(h_instance: ::reaper_low::raw::HINSTANCE, rec: *mut ::reaper_low::raw::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            let static_context = reaper_extension_plugin::static_context();
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
    let email_address = args
        .email_address
        .expect("E-mail address for error reports missing");
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[::reaper_macros::reaper_extension_plugin]
        fn low_level_plugin_main(context: ::reaper_low::ReaperPluginContext) -> Result<(), Box<dyn std::error::Error>> {
            ::reaper_high::ReaperSession::setup_with_defaults(context, #email_address);
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
    /// Support e-mail address which will appear in error reports.
    ///
    /// Necessary for high-level plug-ins only.
    email_address: Option<String>,
}

#[cfg(doctest)]
doc_comment::doctest!("../../../README.md");
