mod mail;

pub use lettre::{
  Message, AsyncSmtpTransport, SmtpTransport, Tokio1Executor, AsyncTransport, message
};

pub use mail::{Email, SmtpEmail};