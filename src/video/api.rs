use crate::utils;

use rocket::{form::Form, get, http::Status, post, serde::json::Json};
use serde::Serialize;
use std::{fs::create_dir_all, io::Write, path::Path};

use super::model::{GenVideoRequestForm, Request, SetEmailRequestForm};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GenVideoRespone {
  code: String,
}

#[post("/api/gen", data = "<data>")]
pub async fn gen_video(
  tx: &rocket::State<tokio::sync::mpsc::Sender<Request>>,
  mut data: Form<GenVideoRequestForm<'_>>,
) -> Result<Json<GenVideoRespone>, Status> {
  // gen random code
  let code = utils::generate_rand_code();
  println!("code : {}", code);

  let dir_path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", &code);
  let video_path = format!("{}/video.mp4", &dir_path);
  let avatar_path = format!("{}/avatar.jpg", &dir_path);

  // save video and avatar to tmp/<code>/
  create_dir_all(&dir_path).map_err(|_| Status::InternalServerError)?;

  data
    .video
    .persist_to(&video_path)
    .await
    .map_err(|_| Status::InternalServerError)?;

  data
    .avatar
    .persist_to(&avatar_path)
    .await
    .map_err(|_| Status::InternalServerError)?;

  // send request to worker
  let request = Request {
    code: code.clone(),
    x: data.x,
    y: data.y,
    shape: data.shape.to_owned(),
    subtitle: data.subtitle,
  };

  tx.try_send(request)
    .map_err(|_| Status::ServiceUnavailable)?;

  Ok(Json(GenVideoRespone { code }))
}

#[post("/api/gen/<code>", data = "<data>")]
pub async fn set_email(code: String, data: Form<SetEmailRequestForm<'_>>) -> Result<(), Status> {
  let email = data.email;

  // check email
  if let Err(_) = email.parse::<lettre::message::Mailbox>() {
    println!("Wrong Email");
    return Err(Status::UnprocessableEntity);
  }

  // write email to tmp/<code>/email.txt
  let file_path = format!(
    "/home/lab603/Documents/slide_talker_backend/tmp/{}/email.txt",
    code
  );

  if let Ok(file) = std::fs::File::create(file_path) {
    let mut writer = std::io::BufWriter::new(file);
    if let Ok(_) = writer.write_all(email.as_bytes()) {
      return Ok(());
    }
  }
  Err(Status::InternalServerError)
}

#[get("/api/gen/<code>")]
pub async fn get_video(code: &str) -> String {
  let path_str = format!("tmp/{}/result.mp4", code);
  let path = Path::new(path_str.as_str());

  // check if tmp/<code>/result.mp4 exists
  if path.exists() {
    format!("http://localhost:8000/download/{}", code)
  } else {
    "not ready".to_owned()
  }
}

#[get("/download/<code>")]
pub async fn download(code: &str) -> Result<rocket::fs::NamedFile, Status> {
  let path_str = format!("tmp/{}/result_subtitle.mp4", code);
  let path = Path::new(path_str.as_str());

  if path.exists() {
    return rocket::fs::NamedFile::open(path)
      .await
      .map_err(|_| Status::InternalServerError);
  }

  let path_str = format!("tmp/{}/result.mp4", code);
  let path = Path::new(path_str.as_str());

  if path.exists() {
    rocket::fs::NamedFile::open(path)
      .await
      .map_err(|_| Status::InternalServerError)
  } else {
    Err(Status::NotFound)
  }
}
