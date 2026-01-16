use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

fn is_bytes(s: &str) -> bool {
  ["Vec<u8>", "Bytes", "bytes::Bytes"].contains(&s) || (s.starts_with("[u8;") && s.ends_with("]"))
}

#[proc_macro_derive(ToSql)]
pub fn sql_struct_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  let fields = match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => &fields.named,
      _ => panic!("ToSqlTrait only supports structs with named fields"),
    },
    _ => panic!("ToSqlTrait only supports structs"),
  };

  let mut kind_matches = Vec::new();
  let mut field_names = Vec::new();
  let mut dump_exprs = Vec::new();

  for field in fields {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let field_type = &field.ty;
    let kind = type_to_kind(field_type);

    kind_matches.push(quote! { #kind });
    field_names.push(quote! { #field_name_str });
    dump_exprs.push(dump_field(field_name, field_type));
  }

  let name_str = name.to_string();

  let mod_name = syn::Ident::new(
    &format!("tosql_linkme_{}", name_str.to_lowercase()),
    name.span(),
  );

  let expanded = quote! {
    impl tosql::ToSqlTrait for #name {
      const META: tosql::Meta = tosql::Meta {
        name: #name_str,
        kind_li: &[#(#field_names),*],
        field_li: &[#(#kind_matches),*],
      };

      fn dump(&self) -> tosql::Bytes {
        use tosql::BufMut;
        let mut buf = tosql::BytesMut::new();
        #(#dump_exprs)*
        buf.freeze()
      }
    }

    tosql::tosql_linkme::turn! {
      mod #mod_name {
        use super::#name;
        use tosql::ToSqlTrait;
        #[tosql::linkme::distributed_slice(tosql::SQL_STRUCT_LI)]
        static I: tosql::SqlStruct = tosql::SqlStruct {
          path: file!(),
          meta: #name::META
        };
      }
    }
  };

  TokenStream::from(expanded)
}

fn type_to_kind(ty: &Type) -> proc_macro2::TokenStream {
  let type_str = quote!(#ty).to_string();
  let type_str = type_str.replace(" ", "");
  match type_str.as_str() {
    "u8" => quote! { tosql::Kind::U8 },
    "i8" => quote! { tosql::Kind::I8 },
    "u16" => quote! { tosql::Kind::U16 },
    "i16" => quote! { tosql::Kind::I16 },
    "u32" => quote! { tosql::Kind::U32 },
    "i32" => quote! { tosql::Kind::I32 },
    "u64" => quote! { tosql::Kind::U64 },
    "i64" => quote! { tosql::Kind::I64 },
    "String" => quote! { tosql::Kind::String },
    "bool" => quote! { tosql::Kind::Bool },
    s if is_bytes(s) => quote! { tosql::Kind::Bytes },
    _ => panic!("tosql_derive unsupported type: {}", type_str),
  }
}

fn dump_field(field_name: &syn::Ident, ty: &Type) -> proc_macro2::TokenStream {
  let type_str = quote!(#ty).to_string();
  let type_str = type_str.replace(" ", "");

  match type_str.as_str() {
    "u8" => quote! {
      buf.put_u8(self.#field_name);
    },
    "bool" => quote! {
      buf.put_u8(self.#field_name as u8);
    },
    "i8" => quote! {
      buf.put_i8(self.#field_name);
    },
    "u16" => quote! {
      buf.put_u16_le(self.#field_name);
    },
    "i16" => quote! {
      buf.put_i16_le(self.#field_name);
    },
    "u32" => quote! {
      {
        let mut bytes = Vec::new();
        tosql::vb::e(self.#field_name as u64, &mut bytes);
        buf.extend_from_slice(&bytes);
      }
    },
    "i32" => quote! {
      buf.put_i32_le(self.#field_name);
    },
    "u64" => quote! {
      {
        let mut bytes = Vec::new();
        tosql::vb::e(self.#field_name, &mut bytes);
        buf.extend_from_slice(&bytes);
      }
    },
    "i64" => quote! {
      buf.put_i64_le(self.#field_name);
    },
    "String" => quote! {
      {
        let data = self.#field_name.as_bytes();
        let mut len_bytes = Vec::new();
        tosql::vb::e(data.len() as u64, &mut len_bytes);
        buf.extend_from_slice(&len_bytes);
        buf.extend_from_slice(data);
      }
    },
    s if is_bytes(s) => quote! {
      {
        let data = &self.#field_name;
        let mut len_bytes = Vec::new();
        tosql::vb::e(data.len() as u64, &mut len_bytes);
        buf.extend_from_slice(&len_bytes);
        buf.extend_from_slice(data);
      }
    },
    _ => panic!("Unsupported type for dump: {}", type_str),
  }
}
