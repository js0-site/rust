use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
  ExprClosure, Ident, Pat,
  parse::{Parse, ParseStream},
  parse_macro_input,
};

struct LinkmeInput {
  closure: ExprClosure,
}

impl Parse for LinkmeInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(Self {
      closure: input.parse()?,
    })
  }
}

/// Generate distributed slice function from closure
/// 从闭包生成分布式切片函数
///
/// # Example
/// ```ignore
/// linkme!(|abc_efg| {
///   println!("id: {}", abc_efg.inner.id);
/// });
/// ```
#[proc_macro]
pub fn linkme(input: TokenStream) -> TokenStream {
  let LinkmeInput { closure } = parse_macro_input!(input as LinkmeInput);

  // Get closure parameter name | 获取闭包参数名
  let param_ident = match closure.inputs.first() {
    Some(Pat::Ident(pat_ident)) => &pat_ident.ident,
    Some(_) => {
      return syn::Error::new_spanned(&closure.inputs, "Expected identifier pattern")
        .to_compile_error()
        .into();
    }
    None => {
      return syn::Error::new_spanned(&closure.inputs, "Expected single parameter")
        .to_compile_error()
        .into();
    }
  };

  // Convert param name to type name (snake_case -> PascalCase) | 参数名转类型名
  let type_name_str = param_ident.to_string().to_case(Case::Pascal);
  let type_name = Ident::new(&type_name_str, param_ident.span());

  let body = &closure.body;

  let fn_name = Ident::new(&format!("linkme_{}", param_ident), param_ident.span());
  quote! {
    #[linkme::distributed_slice(crate::#type_name)]
    fn #fn_name(#param_ident: site_log::ArcArg<crate::#type_name>) -> site_log::JoinHandle {
      site_log::spawn(async move {
        #body
        Ok(())
      })
    }
  }
  .into()
}
