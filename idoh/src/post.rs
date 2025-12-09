use ireq::REQ;
use serde::Deserialize;
use sonic_rs::{JsonValueTrait, Value, from_slice, from_value};

use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct Answer {
  pub name: String,
  pub r#type: u16,
  #[serde(alias = "TTL")]
  pub ttl: u64,
  pub data: String,
}

pub async fn post(doh: &str, query: &str) -> Result<Vec<Answer>> {
  let url = format!("https://{doh}{query}");

  let req = REQ.get(url).header("Accept", "application/dns-json");
  let res = ireq::req(req).await?;

  // log::info!(doh, "{}", String::from_utf8_lossy(&res));
  if let Ok::<Value, _>(json) = xerr::ok!(from_slice(&res))
    && let Some(status) = json.get("Status").as_u64()
    && status == 0
    && let Some(answer_li) = json.get("Answer")
    && let Ok(mut li) = xerr::ok!(from_value::<Vec<Answer>>(answer_li))
  {
    li.iter_mut().for_each(|i| {
      // TXT
      if i.r#type == 16 && i.data.starts_with('"') && i.data.ends_with('"') {
        i.data = i.data[1..i.data.len() - 1].into();
      }
    });
    return Ok(li);
  }

  Err(Error::Doh {
    doh: doh.into(),
    msg: String::from_utf8_lossy(&res).into(),
  })
}
