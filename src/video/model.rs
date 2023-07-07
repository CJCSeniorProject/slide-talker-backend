use rocket::{
  form::{self, Error},
  fs::TempFile,
  http::ContentType,
  FromForm,
};

#[derive(Debug)]
pub struct Request {
  pub code: String,
  pub x: f32,
  pub y: f32,
  pub shape: String,
  pub subtitle: bool,
}

#[derive(FromForm)]
pub struct GenVideoRequestForm<'a> {
  #[field(validate = validate_video())]
  pub video: TempFile<'a>,
  #[field(validate = validate_avatar())]
  pub avatar: TempFile<'a>,
  #[field(validate = validate_range_0_to_1())]
  pub x: f32,
  #[field(validate = validate_range_0_to_1())]
  pub y: f32,
  pub shape: &'a str,
  pub subtitle: bool,
}

fn validate_video<'a>(value: &TempFile<'a>) -> form::Result<'a, ()> {
  if let Some(content_type) = value.content_type() {
    if content_type == &ContentType::MP4 || content_type == &ContentType::MOV {
      return Ok(());
    }
  }
  Err(Error::validation("Invalid file type: MP4 or MOV required").into())
}

fn validate_avatar<'a>(value: &TempFile<'a>) -> form::Result<'a, ()> {
  if let Some(content_type) = value.content_type() {
    if content_type == &ContentType::PNG || content_type == &ContentType::JPEG {
      return Ok(());
    }
  }
  Err(Error::validation("Invalid file type: PNG or JPEG required").into())
}

fn validate_range_0_to_1<'a>(value: &f32) -> form::Result<'a, ()> {
  if *value >= 0.0 && *value <= 1.0 {
    Ok(())
  } else {
    Err(Error::validation("The value must be between 0 and 1").into())
  }
}

#[derive(FromForm)]
pub struct SetEmailRequestForm<'a> {
  pub email: &'a str,
}
