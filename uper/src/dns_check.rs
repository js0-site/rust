use idns::{Answer, QType};
use idoh::{DOH_LI, Doh};
use ver_from_txt::{VerUrlLi, ver_from_txt};

use crate::{Error, Result};

fn extract(project: &str, pre_ver: &[u64; 3], li: &[Answer]) -> Result<Option<VerUrlLi>> {
  for i in li {
    if i.type_id == QType::TXT as u16 {
      return ver_from_txt(project, pre_ver, &i.val).map_err(|e| Error::VerFromTxt(Box::new(e)));
    }
  }
  Ok(None)
}

async fn resolve(
  host: &str,
  _type: &str,
  f: impl FnOnce(&[Answer]) -> Result<Option<VerUrlLi>>,
) -> Result<Option<VerUrlLi>> {
  let doh_clients: Vec<Doh> = DOH_LI.iter().map(|url| Doh::new(*url)).collect();

  for client in doh_clients {
    if let Ok(Some(answers)) = client.query(host, QType::TXT).await {
      return f(&answers);
    }
  }
  Ok(None)
}

pub async fn dns_check(
  project: &str,
  pre_ver: &[u64; 3],
  txt_host_li: &[String],
) -> Result<Option<VerUrlLi>> {
  for host in riter::iter(txt_host_li) {
    let pre_ver = *pre_ver;
    let project = project.to_owned();
    if let Ok(Some(r)) =
      xerr::ok!(resolve(host, "TXT", move |li| extract(&project, &pre_ver, li)).await)
    {
      return Ok(Some(r));
    }
  }
  Ok(None)
}
