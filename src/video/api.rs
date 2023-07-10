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
  log::info!("Generating video");
  // gen random code
  let code = utils::generate_rand_code();
  log::debug!("Generated code : {}", code);

  let dir_path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", &code);
  let video_path = format!("{}/video.mp4", &dir_path);
  let avatar_path = format!("{}/avatar.jpg", &dir_path);

  // save video and avatar to tmp/<code>/
  create_dir_all(&dir_path).map_err(|e| {
    log::error!("Failed to create directory: {}", e);
    Status::InternalServerError
  })?;

  data.video.persist_to(&video_path).await.map_err(|e| {
    log::error!("Failed to persist video: {}", e);
    Status::InternalServerError
  })?;

  data.avatar.persist_to(&avatar_path).await.map_err(|e| {
    log::error!("Failed to persist avatar: {}", e);
    Status::InternalServerError
  })?;

  // send request to worker
  let request = Request {
    code: code.clone(),
    x: data.x,
    y: data.y,
    shape: data.shape.to_owned(),
    subtitle: data.subtitle,
  };

  log::debug!("request={:?}", request);

  tx.try_send(request).map_err(|e| {
    log::error!("Failed to send request to worker: {}", e);
    Status::ServiceUnavailable
  })?;

  log::info!("Video generation request sent for code: {}", code);
  Ok(Json(GenVideoRespone { code }))
}

#[post("/api/gen/<code>", data = "<data>")]
pub async fn set_email(code: String, data: Form<SetEmailRequestForm<'_>>) -> Result<(), Status> {
  log::info!("Setting email for code: {}", code);

  let email = data.email;
  // check email
  if let Err(e) = email.parse::<lettre::message::Mailbox>() {
    log::warn!("Wrong Email: {}", e);
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
      log::info!("Email set for code: {}", code);
      return Ok(());
    }
  }

  log::error!("Failed to write email for code: {}", code);
  Err(Status::InternalServerError)
}

#[get("/api/gen/<code>")]
pub async fn get_video(code: &str) -> String {
  log::info!("Get video for code: {}", code);

  let path_str = format!("tmp/{}/result.mp4", code);
  log::debug!("path={}", path_str);

  let path = Path::new(path_str.as_str());

  // check if tmp/<code>/result.mp4 exists
  if path.exists() {
    log::info!("Video ready for code: {}", code);

    format!("http://localhost:8000/download/{}", code)
  } else {
    log::info!("Video not ready for code: {}", code);

    "not ready".to_owned()
  }
}

#[get("/download/<code>")]
pub async fn download(code: &str) -> Result<rocket::fs::NamedFile, Status> {
  log::info!("Download file for code: {}", code);

  let path_str = format!("tmp/{}/result_subtitle.mp4", code);
  log::debug!("path={}", path_str);

  let path = Path::new(path_str.as_str());

  if path.exists() {
    log::info!("Subtitle file found for code: {}", code);
    return rocket::fs::NamedFile::open(path).await.map_err(|e| {
      log::error!("Failed to open subtitle file for code: {}: {}", code, e);
      Status::InternalServerError
    });
  }

  let path_str = format!("tmp/{}/result.mp4", code);
  log::debug!("path={}", path_str);

  let path = Path::new(path_str.as_str());

  if path.exists() {
    log::info!("Video file found for code: {}", code);
    rocket::fs::NamedFile::open(path).await.map_err(|e| {
      log::error!("Failed to open video file for code: {}: {}", code, e);
      Status::InternalServerError
    })
  } else {
    log::warn!("File not found for code: {}", code);
    Err(Status::NotFound)
  }
}
