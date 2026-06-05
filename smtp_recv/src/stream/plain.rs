use tokio::{
  io::{BufReader, ReadHalf, WriteHalf},
  net::TcpStream,
};

use super::ReaderWriter;

/// 明文TCP流
pub struct PlainStream {
  pub reader: BufReader<ReadHalf<TcpStream>>,
  pub writer: WriteHalf<TcpStream>,
}

impl ReaderWriter for PlainStream {
  type Reader = BufReader<ReadHalf<TcpStream>>;
  type Writer = WriteHalf<TcpStream>;

  fn reader(&mut self) -> &mut Self::Reader {
    &mut self.reader
  }

  fn writer(&mut self) -> &mut Self::Writer {
    &mut self.writer
  }
}
