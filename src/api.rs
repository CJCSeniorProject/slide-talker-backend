use crate::{
  database,
  model::{constant::*, email, task, video, worker},
  utils,
};
use rocket::{form::Form, get, http::Status, post, serde::json::Json};

#[post("/api/gen", data = "<data>")]
pub async fn gen_video(
  tx: &rocket::State<tokio::sync::mpsc::Sender<worker::Request>>,
  mut data: Form<video::Request<'_>>,
) -> Result<Json<video::Response>, Status> {
  log::info!("Generating video");
  // gen random code
  let mut code = utils::generate_rand_code();
  log::debug!("Generated code : {}", code);
  while let Some(true) = database::check_code_exists(&code).ok() {
    code = utils::generate_rand_code();
  }
  utils::create_code_dir(&code).map_err(|_| Status::InternalServerError)?;
  let video_path =
    utils::create_file(&code, VIDEO_FILE).map_err(|_| Status::InternalServerError)?;
  let avatar_path =
    utils::create_file(&code, AVATAR_FILE).map_err(|_| Status::InternalServerError)?;

  data.video.persist_to(video_path).await.map_err(|e| {
    log::error!("Failed to persist video: {:?}", e);
    Status::InternalServerError
  })?;

  data.avatar.persist_to(avatar_path).await.map_err(|e| {
    log::error!("Failed to persist avatar: {:?}", e);
    Status::InternalServerError
  })?;

  if let Err(e) = database::insert_task(&code) {
    log::error!("{}", e);
    return Err(Status::InternalServerError);
  };

  // send request to worker
  let request = worker::Request {
    code: code.clone(),
    x: data.x,
    y: data.y,
    shape: data.shape.to_owned(),
    subtitle: data.subtitle,
  };

  log::debug!("request={:?}", request);

  tx.try_send(request).map_err(|e| {
    log::error!("Failed to send request to worker: {:?}", e);
    Status::ServiceUnavailable
  })?;

  log::info!("Video generation request sent for code: {}", code);
  Ok(Json(video::Response { code }))
}

#[post("/api/gen/<code>", data = "<data>")]
pub async fn set_email(code: &str, data: Form<email::Request>) -> Result<(), Status> {
  log::info!("Setting email for code: {}", code);

  let email = &data.email;
  // check email
  if let Err(e) = email.parse::<lettre::message::Mailbox>() {
    log::warn!("Wrong Email: {:?}", e);
    Err(Status::UnprocessableEntity)?
  }

  if let Err(e) = database::update_task_email(code, email) {
    log::error!("Failed to write email: {:?}", e);
    Err(Status::InternalServerError)?
  }
  Ok(())
}

#[get("/api/gen/<code>")]
pub async fn get_video(code: &str) -> Result<(), Status> {
  log::info!("Get video for code: {}", code);

  match database::get_task_status(code) {
    Ok(status) => {
      log::debug!("Video status={}", status.to_string());
      match status {
        task::Status::Fail => Err(Status::InternalServerError),
        task::Status::Finish => Ok(()),
        task::Status::Processing => Err(Status::new(499)),
      }
    }
    Err(e) => {
      log::error!("{}", e);
      Err(Status::InternalServerError)
    }
  }
}

#[get("/download/<code>")]
pub async fn download(code: &str) -> Result<rocket::fs::NamedFile, Status> {
  log::info!("Download file for code: {}", code);

  if let Ok(file_path) = get_file_path(code, RESULT_WITH_SUBS_FILE) {
    log::info!("Result with subtitles file found for code: {}", code);
    return rocket::fs::NamedFile::open(file_path).await.map_err(|e| {
      log::error!(
        "Failed to open result with subtitles file for code: {}: {:?}",
        code,
        e
      );
      Status::InternalServerError
    });
  }

  if let Ok(file_path) = get_file_path(code, RESULT_FILE) {
    log::info!("Result file found for code: {}", code);
    rocket::fs::NamedFile::open(file_path).await.map_err(|e| {
      log::error!("Failed to open result file for code: {}: {:?}", code, e);
      Status::InternalServerError
    })
  } else {
    log::warn!("File not found for code: {}", code);
    Err(Status::NotFound)
  }
}

#[get("/file/<code>/<filename>")]
pub fn get_file_path(code: &str, filename: &str) -> Result<String, Status> {
  if let Ok(path) = utils::get_file_path(code, filename) {
    return Ok(path);
  }
  Err(Status::NotFound)
}
