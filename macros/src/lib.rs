use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn reaper_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    let tokens = quote! {
        #[no_mangle]
        extern "C" fn ReaperPluginEntry(h_instance: low_level::HINSTANCE, rec: *mut low_level::reaper_plugin_info_t) -> c_int {
            ::reaper_rs::low_level::bootstrap_reaper_plugin(h_instance, rec, #name)
        }

        #input
    };
    tokens.into()
}