use serde::{Serialize, Serializer, ser::SerializeTuple};

mod serde_error {
  use super::*;

  pub struct IdohErrorWrapper<'a>(pub &'a idot::Error);
  impl<'a> Serialize for IdohErrorWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
      serialize_idot_error(self.0, serializer)
    }
  }

  pub struct MailErrorWrapper<'a>(pub &'a mail_send::Error);
  impl<'a> Serialize for MailErrorWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
      serialize_mail_error(self.0, serializer)
    }
  }

  pub struct SmtpProtoResponseWrapper<'a>(pub &'a smtp_proto::Response<String>);
  impl<'a> Serialize for SmtpProtoResponseWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
      serialize_smtp_proto_response(self.0, serializer)
    }
  }

  pub fn serialize_idot_error<S>(err: &idot::Error, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&err.to_string())
  }

  pub fn serialize_mail_error<S>(err: &mail_send::Error, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&err.to_string())
  }

  pub fn serialize_smtp_proto_response<S>(
    resp: &smtp_proto::Response<String>,
    serializer: S,
  ) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&resp.to_string())
  }
}

// 邮件拒绝错误 / Email rejection error
#[derive(Debug)]
pub struct Reject {
  pub email: String,
  pub server: String,
  pub response: smtp_proto::Response<String>,
}

// 发送错误 / Send error
#[derive(Debug)]
pub struct SendErr {
  pub email: String,
  pub server: String,
  pub error: mail_send::Error,
}

#[derive(Debug)]
pub enum Error {
  DkimInit(String),
  DnsResolveFailed(String, idot::Error),
  MxIsEmpty(String),
  Reject(Reject),
  SendErr(SendErr),
  SmtpAllFailed(String, mail_send::Error),
  // Received 头部过多 / Too many Received headers
  TooManyReceived(String),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Error::DkimInit(host) => {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element("DkimInit")?;
        seq.serialize_element(host)?;
        seq.end()
      }
      Error::DnsResolveFailed(host, err) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element("DnsResolveFailed")?;
        seq.serialize_element(host)?;
        seq.serialize_element(&serde_error::IdohErrorWrapper(err))?;
        seq.end()
      }
      Error::MxIsEmpty(host) => {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element("MxIsEmpty")?;
        seq.serialize_element(host)?;
        seq.end()
      }
      Error::Reject(info) => {
        let mut seq = serializer.serialize_tuple(4)?;
        seq.serialize_element("reject")?;
        seq.serialize_element(&info.email)?;
        seq.serialize_element(&info.server)?;
        seq.serialize_element(&serde_error::SmtpProtoResponseWrapper(&info.response))?;
        seq.end()
      }
      Error::SendErr(info) => {
        let mut seq = serializer.serialize_tuple(4)?;
        seq.serialize_element("SendErr")?;
        seq.serialize_element(&info.email)?;
        seq.serialize_element(&info.server)?;
        seq.serialize_element(&serde_error::MailErrorWrapper(&info.error))?;
        seq.end()
      }
      Error::SmtpAllFailed(host, errs) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element("SmtpAllFailed")?;
        seq.serialize_element(host)?;
        seq.serialize_element(&serde_error::MailErrorWrapper(errs))?;
        seq.end()
      }
      Error::TooManyReceived(sender) => {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element("TooManyReceived")?;
        seq.serialize_element(sender)?;
        seq.end()
      }
    }
  }
}
