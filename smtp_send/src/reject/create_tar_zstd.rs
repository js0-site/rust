pub fn create_tar_zstd(mail_bin: &[u8], result_yml: &str) -> Vec<u8> {
  let mut tar_builder = tar::Builder::new(Vec::new());

  [
    ("mail.bin", mail_bin as &[u8]),
    ("result.yml", result_yml.as_bytes()),
  ]
  .into_iter()
  .for_each(|(path, data)| {
    let mut header = tar::Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, data).unwrap();
  });

  let tar_data = tar_builder.into_inner().unwrap();

  zstd::encode_all(tar_data.as_slice(), 3).unwrap()
}
