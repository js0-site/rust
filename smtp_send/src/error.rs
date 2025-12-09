use serde::{Serialize, Serializer, ser::SerializeTuple};

mod serde_error {
  use super::*;

  pub struct IdohErrorWrapper<'a>(pub &'a idoh::Error);
  impl<'a> Serialize for IdohErrorWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
      serialize_idoh_error(self.0, serializer)
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

  pub fn serialize_idoh_error<S>(err: &idoh::Error, serializer: S) -> Result<S::Ok, S::Error>
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

#[derive(Debug)]
pub enum Error {
  DnsResolveFailed(String, idoh::Error),
  MxIsEmpty(String),
  Reject(String, smtp_proto::Response<String>),
  SendErr(String, mail_send::Error),
  SmtpAllFailed(String, mail_send::Error),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
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
      Error::Reject(host, resp) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element("reject")?;
        seq.serialize_element(host)?;
        seq.serialize_element(&serde_error::SmtpProtoResponseWrapper(resp))?;
        seq.end()
      }
      Error::SendErr(host, errs) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element("SendErr")?;
        seq.serialize_element(host)?;
        seq.serialize_element(&serde_error::MailErrorWrapper(errs))?;
        seq.end()
      }
      Error::SmtpAllFailed(host, errs) => {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element("SmtpAllFailed")?;
        seq.serialize_element(host)?;
        seq.serialize_element(&serde_error::MailErrorWrapper(errs))?;
        seq.end()
      }
    }
  }
}
