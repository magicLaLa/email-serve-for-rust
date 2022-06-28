use lettre::{
    transport::smtp::{authentication::Credentials, AsyncSmtpTransportBuilder, Error},
    AsyncSmtpTransport, SmtpTransport, Tokio1Executor,
};

pub trait SmtpEmail {
    fn builder(&self) -> Result<SmtpTransport, Error>;
    fn async_builder(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, Error>;
}

/// Email
#[derive(Debug, Clone)]
pub struct Email {
    pub smtp_serve: String,
    pub user_name: String,
    pub password: String,
}

impl Email {
    /// 创建 Email 结构体
    pub fn new(smtp_serve: String, user_name: String, password: String) -> Self {
        Self {
            smtp_serve,
            user_name,
            password,
        }
    }
}

impl SmtpEmail for Email {
    fn builder(&self) -> Result<SmtpTransport, Error> {
        match SmtpTransport::relay(self.smtp_serve.as_str()) {
            Ok(res) => {
                let cerd = Credentials::new(self.user_name.clone(), self.password.clone());
                Ok(res.credentials(cerd).build())
            }
            Err(e) => Err(e),
        }
    }

    fn async_builder(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, Error> {
        let async_smtp: Result<AsyncSmtpTransportBuilder, Error> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(self.smtp_serve.as_str());
        match async_smtp {
            Ok(res) => {
                let cerd = Credentials::new(self.user_name.clone(), self.password.clone());
                Ok(res.credentials(cerd).build())
            }
            Err(e) => Err(e),
        }
    }
}
