use jdb_xorf::{Bf, Bf8};

fn main() {
  // 1. Manually construct/load a filter.
  // In a real scenario, this might come from `bitcode::decode` or keys known to be strings.
  // Here we use a temporary helper to build it from &str, keeping in mind the keys are hashed.
  let keys = vec!["apple", "banana", "cherry"];
  // Build a normal Bf<&str> first to get the underlying raw filter
  let temp_filter: Bf<&str, Bf8> = Bf::from(&keys);

  // 2. Wrap the raw filter into a Bf<str> (unsized!)
  // This allows the type to semantically represent "Filter of strings" rather than "Filter of references".
  // Note: Bf::from cannot be used directly with Bf<str> because [str] is unsized.
  let str_filter: Bf<str, Bf8> = temp_filter.into();

  // 3. Query
  // Query with &str. Since T=str, and str: Borrow<str>, key is &str.
  // This demonstrates using the Unsized T support.
  assert!(str_filter.has("apple"));
  assert!(str_filter.has("banana"));
  assert!(!str_filter.has("durian"));

  println!("Bf<str> example works: 'apple' found, 'durian' not found.");
}
