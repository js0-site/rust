// ============================================
// Tests for user_host
// ============================================

#[test]
fn test_user_host_normal() {
  let (user, host) = xmail::user_host("hello@example.com").unwrap();
  assert_eq!(user, "hello");
  assert_eq!(host, "example.com");
}

#[test]
fn test_user_host_empty_user() {
  assert!(xmail::user_host("@example.com").is_none());
}

#[test]
fn test_user_host_no_at_symbol() {
  assert!(xmail::user_host("example.com").is_none());
}

#[test]
fn test_user_host_only_at_symbol() {
  assert!(xmail::user_host("@").is_none());
}

#[test]
fn test_user_host_empty_string() {
  assert!(xmail::user_host("").is_none());
}

#[test]
fn test_user_host_multiple_at_symbols() {
  // Should split at the first @
  let (user, host) = xmail::user_host("user@domain.com@extra").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "domain.com@extra");
}

#[test]
fn test_user_host_no_dot() {
  assert!(xmail::user_host("user@localhost").is_none());
}

#[test]
fn test_user_host_empty_host_prefix() {
  assert!(xmail::user_host("user@.com").is_none());
}

#[test]
fn test_user_host_empty_host_suffix() {
  assert!(xmail::user_host("user@domain.").is_none());
}

#[test]
fn test_user_host_dot_only() {
  assert!(xmail::user_host("user@.").is_none());
}

#[test]
fn test_user_host_with_subdomain() {
  let (user, host) = xmail::user_host("admin@mail.example.com").unwrap();
  assert_eq!(user, "admin");
  assert_eq!(host, "mail.example.com");
}

// ============================================
// Tests for norm_user_host
// ============================================

#[test]
fn test_norm_user_host_lowercase() {
  let (user, host) = xmail::norm_user_host("USER@EXAMPLE.COM").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "example.com");
}

#[test]
fn test_norm_user_host_trim_whitespace() {
  let (user, host) = xmail::norm_user_host("  user@example.com  ").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "example.com");
}

#[test]
fn test_norm_user_host_punycode_decode() {
  // xn--yfro4i67o is punycode for 新加坡
  let (user, host) = xmail::norm_user_host("user@site.xn--yfro4i67o").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "site.新加坡");
}

#[test]
fn test_norm_user_host_mixed_case_and_whitespace() {
  let (user, host) = xmail::norm_user_host("  HeLLo@WoRLd.CoM  ").unwrap();
  assert_eq!(user, "hello");
  assert_eq!(host, "world.com");
}

#[test]
fn test_norm_user_host_multiple_subdomains() {
  let (user, host) = xmail::norm_user_host("user@a.b.c.example.com").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "a.b.c.example.com");
}

#[test]
fn test_norm_user_host_punycode_multiple_parts() {
  // Multiple punycode parts
  let (user, host) = xmail::norm_user_host("user@xn--yfro4i67o.xn--yfro4i67o").unwrap();
  assert_eq!(user, "user");
  assert_eq!(host, "新加坡.新加坡");
}

#[test]
fn test_norm_user_host_none() {
  assert!(xmail::norm_user_host("invalid").is_none());
}

// ============================================
// Tests for norm
// ============================================

#[test]
fn test_norm_basic() {
  assert_eq!(xmail::norm("user@example.com").unwrap(), "user@example.com");
}

#[test]
fn test_norm_uppercase() {
  assert_eq!(xmail::norm("USER@EXAMPLE.COM").unwrap(), "user@example.com");
}

#[test]
fn test_norm_with_whitespace() {
  assert_eq!(
    xmail::norm("  user@example.com  ").unwrap(),
    "user@example.com"
  );
}

#[test]
fn test_norm_punycode() {
  assert_eq!(
    xmail::norm("  3Ti@Site.xn--yfro4i67o ").unwrap(),
    "3ti@site.新加坡"
  );
}

#[test]
fn test_norm_complex() {
  assert_eq!(
    xmail::norm("  ADMIN@Mail.EXAMPLE.xn--yfro4i67o  ").unwrap(),
    "admin@mail.example.新加坡"
  );
}

#[test]
fn test_norm_none() {
  assert!(xmail::norm("invalid").is_none());
}

// ============================================
// Tests for norm_tld (requires tld feature)
// ============================================

#[cfg(feature = "tld")]
#[test]
fn test_norm_tld_basic() {
  let (mail, tld) = xmail::norm_tld("user@example.com").unwrap();
  assert_eq!(mail, "user@example.com");
  assert_eq!(tld, "example.com");
}

#[cfg(feature = "tld")]
#[test]
fn test_norm_tld_with_subdomain() {
  let (mail, tld) = xmail::norm_tld("user@mail.example.com").unwrap();
  assert_eq!(mail, "user@mail.example.com");
  assert_eq!(tld, "example.com");
}

#[cfg(feature = "tld")]
#[test]
fn test_norm_tld_punycode() {
  let (mail, tld) = xmail::norm_tld("  3Ti@Site.aA.xn--yfro4i67o ").unwrap();
  assert_eq!(mail, "3ti@site.aa.新加坡");
  assert_eq!(tld, "aa.新加坡");
}

#[cfg(feature = "tld")]
#[test]
fn test_norm_tld_uppercase() {
  let (mail, tld) = xmail::norm_tld("USER@EXAMPLE.ORG").unwrap();
  assert_eq!(mail, "user@example.org");
  assert_eq!(tld, "example.org");
}

#[cfg(feature = "tld")]
#[test]
fn test_norm_tld_none() {
  assert!(xmail::norm_tld("invalid").is_none());
}
