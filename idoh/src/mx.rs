use aok::Result;

use crate::{record_type, resolve};

/// MX record structure
#[derive(Debug, Clone)]
pub struct Mx {
  /// Priority of the mail server (lower values have higher priority)
  pub priority: u16,
  /// Mail server hostname
  pub server: String,
  /// Time to live in seconds
  pub ttl: u64,
}

/// Query MX records for a domain
///
/// # Example
///
/// ```ignore
/// use idoh::mx;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///   let mx_records = mx("gmail.com").await?;
///   for record in mx_records {
///     println!("Priority: {}, Server: {}", record.priority, record.server);
///   }
///   Ok(())
/// }
/// ```
pub async fn mx(domain: impl AsRef<str>) -> Result<Vec<Mx>> {
  resolve(domain, "MX", |li| {
    let mut r = Vec::with_capacity(li.len());
    for i in li {
      if i.r#type == record_type::MX {
        // MX data format: "priority server"
        // Example: "10 mail.example.com."
        if let Some((priority_str, server)) = i.data.split_once(' ')
          && let Ok(priority) = priority_str.parse::<u16>()
        {
          r.push(Mx {
            priority,
            server: server.trim_end_matches('.').to_string(),
            ttl: i.ttl,
          });
        }
      }
    }
    Ok(Some(r))
  })
  .await
}
