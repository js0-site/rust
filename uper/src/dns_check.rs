use idoh::Answer;
use ver_from_txt::{VerUrlLi, ver_from_txt};

use crate::Result;

fn extract(project: &str, pre_ver: &[u64; 3], li: &[Answer]) -> idoh::Result<Option<VerUrlLi>> {
  for i in li {
    if i.r#type == idoh::record_type::TXT {
      return ver_from_txt(project, pre_ver, &i.data)
        .map_err(|e| idoh::Error::InvalidData(format!("{}", e).into()));
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
      xerr::ok!(idoh::resolve(host, "TXT", move |li| extract(&project, &pre_ver, li)).await)
    {
      return Ok(Some(r));
    }
  }
  Ok(None)
}
