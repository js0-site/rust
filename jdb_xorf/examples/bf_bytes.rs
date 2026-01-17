use jdb_xorf::{Bf, Bf8};

fn main() {
    let data: Vec<&[u8]> = vec![b"hello", b"world"];
    
    // 1. Build a normal Bf<&[u8]> 
    let temp_filter: Bf<&[u8], Bf8> = Bf::from(&data);

    // 2. Wrap into Bf<[u8]>
    let bytes_filter: Bf<[u8], Bf8> = temp_filter.into();

    // 3. Query
    assert!(bytes_filter.has(b"hello".as_slice()));
    assert!(bytes_filter.has(b"world".as_slice()));
    assert!(!bytes_filter.has(b"foo".as_slice()));

    println!("Bf<[u8]> example works: 'hello' found, 'foo' not found.");
}
