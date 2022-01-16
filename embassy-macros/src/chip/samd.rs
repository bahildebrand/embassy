use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_samd_path = embassy_prefix.append("embassy_samd").path();

    quote!(
        let p = #embassy_samd_path::init(#config);
    )
}
