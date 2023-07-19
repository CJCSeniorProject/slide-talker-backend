use rocket::FromForm;
use serde::{Deserialize, Serialize};

#[derive(FromForm, Serialize, Deserialize)]
pub struct Request {
  pub email: String,
}
