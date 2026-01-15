use std::io;

use tar::{Builder, Header};

pub fn create(mail: &[u8], yml: &str) -> io::Result<Vec<u8>> {
  let mut builder = Builder::new(Vec::new());

  for (path, data) in [("mail.bin", mail), ("result.yml", yml.as_bytes())] {
    let mut header = Header::new_gnu();
    header.set_path(path)?;
    // usize -> u64 在 64 位系统上不会溢出 / usize -> u64 won't overflow on 64-bit
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, data)?;
  }

  let tar = builder.into_inner()?;
  zstd::encode_all(tar.as_slice(), 3)
}
