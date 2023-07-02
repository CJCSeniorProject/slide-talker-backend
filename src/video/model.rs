use rocket::fs::TempFile;

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
  pub video: TempFile<'a>,
  pub avatar: TempFile<'a>,
  pub x: f32,
  pub y: f32,
  pub shape: &'a str,
  pub subtitle: bool,
}

#[derive(FromForm)]
pub struct SetEmailRequestForm<'a> {
  pub email: &'a str,
}
