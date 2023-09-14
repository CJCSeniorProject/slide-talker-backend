use rocket::{
  form::{self, Error},
  fs::TempFile,
  http::ContentType,
  FromForm,
};

#[derive(FromForm, Debug)]
pub struct Request<'a> {
  #[field(validate = validate_video())]
  pub video: TempFile<'a>,
  #[field(validate = validate_avatar())]
  pub avatar: TempFile<'a>,
  #[field(validate = validate_range_0_to_1())]
  pub x: f32,
  #[field(validate = validate_range_0_to_1())]
  pub y: f32,
  pub shape: String,
  #[field(default = true)]
  pub remove_bg: bool,
  #[field(default = false)]
  pub subtitle: bool,
}

fn validate_video<'a>(value: &TempFile<'a>) -> form::Result<'a, ()> {
  if let Some(content_type) = value.content_type() {
    if content_type == &ContentType::MP4 || content_type == &ContentType::MOV {
      return Ok(());
    }
  }
  log::warn!("Invalid file type: MP4 or MOV required");
  Err(Error::validation("Invalid file type: MP4 or MOV required").into())
}

fn validate_avatar<'a>(value: &TempFile<'a>) -> form::Result<'a, ()> {
  if let Some(content_type) = value.content_type() {
    if content_type == &ContentType::PNG || content_type == &ContentType::JPEG {
      return Ok(());
    }
  }
  log::warn!("Invalid file type: PNG or JPEG required");
  Err(Error::validation("Invalid file type: PNG or JPEG required").into())
}

fn validate_range_0_to_1<'a>(value: &f32) -> form::Result<'a, ()> {
  if *value >= 0.0 && *value <= 1.0 {
    Ok(())
  } else {
    log::warn!("The value must be between 0 and 1");
    Err(Error::validation("The value must be between 0 and 1").into())
  }
}
