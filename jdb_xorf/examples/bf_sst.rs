#[cfg(feature = "bitcode")]
mod example {
    use bitcode::{Decode, Encode};
    use jdb_xorf::{Bf, Bf8};

    #[derive(Encode, Decode)]
    pub struct Sst {
        pub xorf: Bf<[u8], Bf8>,
    }

    pub fn run() {
        let keys: Vec<&[u8]> = vec![b"key1", b"key2", b"key3"];

        // 1. Build filter (Bf<&[u8]>)
        let temp_filter: Bf<&[u8], Bf8> = Bf::from(&keys);

        // 2. Convert to Bf<[u8]> and put into Sst
        let sst = Sst {
            xorf: temp_filter.into(),
        };

        // 3. Serialize
        // Use default bitcode encoding
        let encoded: Vec<u8> = bitcode::encode(&sst);
        println!("Serialized size: {} bytes", encoded.len());

        // 4. Deserialize
        let decoded: Sst = bitcode::decode(&encoded).expect("Decoding failed");

        // 5. Verify
        let filter = decoded.xorf;
        assert!(filter.has(b"key1".as_slice()));
        assert!(filter.has(b"key2".as_slice()));
        assert!(filter.has(b"key3".as_slice()));
        assert!(!filter.has(b"key4".as_slice()));

        println!("Sst example works: bits preserved and binary query successful.");
    }
}

fn main() {
    #[cfg(feature = "bitcode")]
    example::run();

    #[cfg(not(feature = "bitcode"))]
    println!("Please run with --features bitcode");
}
