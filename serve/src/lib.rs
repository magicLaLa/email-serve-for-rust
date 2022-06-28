mod lib {
  pub mod email_mod;
}

pub use lib::email_mod::router as Email_Serve;