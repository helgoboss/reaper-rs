use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn low_level_reaper_plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    let main_function = syn::parse_macro_input!(input as syn::ItemFn);
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[no_mangle]
        extern "C" fn ReaperPluginEntry(h_instance: ::reaper_rs::low_level::HINSTANCE, rec: *mut ::reaper_rs::low_level::reaper_plugin_info_t) -> ::std::os::raw::c_int {
            ::reaper_rs::low_level::bootstrap_reaper_plugin(h_instance, rec, #main_function_name)
        }

        #main_function
    };
    tokens.into()
}

#[derive(Debug, FromMeta)]
struct ReaperPluginMacroArgs {
    email_address: String,
}

#[proc_macro_attribute]
pub fn reaper_plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let main_function = syn::parse_macro_input!(input as syn::ItemFn);
    let args = match ReaperPluginMacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => { return e.write_errors().into(); }
    };
    let email_address = args.email_address;
    let main_function_name = &main_function.sig.ident;
    let tokens = quote! {
        #[::reaper_rs_macros::low_level_reaper_plugin]
        fn low_level_main(context: ::reaper_rs::low_level::ReaperPluginContext) -> Result<(), &'static str> {
            ::reaper_rs::high_level::setup_all_with_defaults(context, #email_address);
            #main_function_name()
        }

        #main_function
    };
    tokens.into()
}