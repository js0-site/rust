#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;

pub use error::{Error, Result};
pub use kind2sql::Kind;

#[derive(Debug)]
pub struct Meta {
  pub name: String,
  pub kind_li: Vec<String>,
  pub field_li: Vec<Kind>,
}

impl Meta {
  pub fn load(bin: &[u8]) -> Result<Self> {
    let bin_len = bin.len();
    if bin_len < 2 {
      return Err(Error::EmptyData);
    }

    let name_len = bin[0] as usize;

    let name = String::from_utf8_lossy(&bin[1..name_len + 1]).into();
    let mut pos = name_len + 1;
    let count = bin[pos] as usize;
    pos += 1;

    let mut kind_li = Vec::with_capacity(count);

    // Parse null-terminated strings
    for _ in 0..count {
      let start = pos;
      while pos < bin_len && bin[pos] != 0 {
        pos += 1;
      }
      if pos >= bin_len {
        return Err(Error::TruncatedFieldNames);
      }

      let field_name = String::from_utf8(bin[start..pos].to_vec())?;
      kind_li.push(field_name);
      pos += 1; // skip null terminator
    }

    // Parse field types (Kind enum values)
    let field_count = bin_len - pos;
    let mut field_li = Vec::with_capacity(field_count);
    while pos < bin_len {
      let kind_value = bin[pos];
      // Convert u8 to Kind enum using TryFrom
      let kind = Kind::try_from(kind_value).map_err(|_| Error::InvalidKindValue(kind_value))?;
      field_li.push(kind);
      pos += 1;
    }

    Ok(Self {
      name,
      kind_li,
      field_li,
    })
  }
}
