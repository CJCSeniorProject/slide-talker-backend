use rocket::{
  form::{self, Error},
  FromForm,
};
use serde::{Deserialize, Serialize};

#[derive(FromForm, Serialize, Deserialize)]
pub struct Request {
  #[field(validate = validate_email())]
  pub email: String,
}

fn validate_email<'a>(email: &String) -> form::Result<'a, ()> {
  if let Err(e) = email.parse::<lettre::message::Mailbox>() {
    log::warn!("Wrong Email: {:?}", e);

    return Err(Error::validation("Invalid email").into());
  }
  Ok(())
}
