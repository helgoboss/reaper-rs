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
        reaper_low::dll_main!();
        reaper_low::swell_dll_main!();

        /// Entry point for the REAPER extension plug-in.
        ///
        /// This is called by REAPER at startup time.
        #[no_mangle]
        unsafe extern "C" fn ReaperPluginEntry(h_instance: ::reaper_low::raw::HINSTANCE, rec: *mut ::reaper_low::raw::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            let static_context = reaper_low::static_plugin_context();
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
    let update_url = args.update_url.expect("update_url missing");
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[::reaper_macros::reaper_extension_plugin]
        fn low_level_plugin_main(context: ::reaper_low::PluginContext) -> Result<(), Box<dyn std::error::Error>> {
            let plugin_info = ::reaper_high::PluginInfo {
                plugin_name: #plugin_name.to_string(),
                plugin_version: #plugin_version.to_string(),
                plugin_version_long: #plugin_version.to_string(),
                support_email_address: #support_email_address.to_string(),
                update_url: #update_url.to_string(),
            };
            ::reaper_high::Reaper::setup_with_defaults(context, plugin_info)
                .map_err(|_| "attempt to setup reaper_high instance more than once")?;
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
    /// URL that will be shown in error reports in order to animate the user to first try
    /// again with the latest update.
    ///
    /// Necessary for high-level plug-in.
    update_url: Option<String>,
}

#[cfg(doctest)]
doc_comment::doctest!("../../../README.md");
