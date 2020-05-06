use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

/// Macro for easily bootstrapping a REAPER extension plug-in.
///
/// Use the macro like this:
/// ```
/// use std::error::Error;
/// use reaper_rs_macros::reaper_extension_plugin;
/// use reaper_rs_low::ReaperPluginContext;
///
/// #[reaper_extension_plugin]
/// fn main(context: &ReaperPluginContext) -> Result<(), Box<dyn Error>> {
///     let reaper = reaper_rs_medium::Reaper::load(context);
///     reaper.show_console_msg("Hello world from *reaper-rs* medium-level API!");
///     Ok(())
/// }
/// ```
///
/// If you want to start with a preconfigured high-level `Reaper` instance right away, use the macro
/// like this:
///
/// ```
/// use std::error::Error;
/// use reaper_rs_macros::reaper_extension_plugin;
/// use reaper_rs_high::Reaper;
///
/// #[reaper_extension_plugin(email_address = "support@example.org")]
/// fn main() -> Result<(), Box<dyn Error>> {
///     let reaper = Reaper::get();
///     reaper.show_console_msg("Hello world from *reaper-rs* high-level API!");
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
        #[no_mangle]
        extern "C" fn ReaperPluginEntry(h_instance: ::reaper_rs_low::raw::HINSTANCE, rec: *mut ::reaper_rs_low::raw::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            ::reaper_rs_low::bootstrap_extension_plugin(h_instance, rec, #main_function_name)
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
        #[::reaper_rs_macros::reaper_extension_plugin]
        fn low_level_main(context: &::reaper_rs_low::ReaperPluginContext) -> Result<(), Box<dyn std::error::Error>> {
            ::reaper_rs_high::Reaper::setup_with_defaults(context, #email_address);
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
