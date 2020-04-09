use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn low_level_reaper_extension_plugin(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let main_function = syn::parse_macro_input!(input as syn::ItemFn);
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[no_mangle]
        extern "C" fn ReaperPluginEntry(h_instance: ::reaper_rs::low_level::raw::HINSTANCE, rec: *mut ::reaper_rs::low_level::raw::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            ::reaper_rs::low_level::bootstrap_extension_plugin(h_instance, rec, #main_function_name)
        }

        #main_function
    };
    tokens.into()
}

#[derive(Debug, FromMeta)]
struct ReaperExtensionPluginMacroArgs {
    email_address: String,
}

#[proc_macro_attribute]
pub fn reaper_extension_plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let main_function = syn::parse_macro_input!(input as syn::ItemFn);
    let args = match ReaperExtensionPluginMacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };
    let email_address = args.email_address;
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[::reaper_rs_macros::low_level_reaper_extension_plugin]
        fn low_level_main(context: &::reaper_rs::low_level::ReaperPluginContext) -> Result<(), Box<dyn std::error::Error>> {
            ::reaper_rs::high_level::setup_all_with_defaults(context, #email_address);
            #main_function_name()
        }

        #main_function
    };
    tokens.into()
}
