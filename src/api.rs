use crate::{database, model::constant::*, model::*, utils::*};
use rocket::{form::Form, fs::NamedFile, get, http::Status, post, serde::json::Json};

#[post("/api/gen", data = "<data>")]
pub async fn gen_video(
  tx: &rocket::State<tokio::sync::mpsc::Sender<worker::Request>>,
  mut data: Form<video::Request<'_>>,
) -> Result<Json<video::Response>, Status> {
  log::info!("Generating video");

  // gen random code
  let mut code = generate_rand_code();
  while let Some(true) = database::check_code_exists(&code).ok() {
    code = generate_rand_code();
  }
  log::debug!("Generated code : {}", code);

  create_code_dir(&code).map_err(|_| Status::InternalServerError)?;
  let video_path = create_file(&code, VIDEO_FILE).map_err(|_| Status::InternalServerError)?;
  let avatar_path = create_file(&code, AVATAR_FILE).map_err(|_| Status::InternalServerError)?;

  handle(data.video.persist_to(video_path).await, "Persisting video")
    .map_err(|_| Status::InternalServerError)?;

  handle(
    data.avatar.persist_to(avatar_path).await,
    "Persisting avatar",
  )
  .map_err(|_| Status::InternalServerError)?;

  handle(
    database::insert_task(&code),
    &format!("Inserting task for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;

  // send request to worker
  let request = worker::Request {
    code: code.clone(),
    x: data.x,
    y: data.y,
    shape: data.shape.to_owned(),
    subtitle: data.subtitle,
  };
  log::debug!("request={:?}", request);

  handle(tx.try_send(request), "Sending request to worker")
    .map_err(|_| Status::ServiceUnavailable)?;

  log::info!("Video generat  ion request sent for code: {}", code);
  Ok(Json(video::Response { code }))
}

#[post("/api/gen/<code>", data = "<data>")]
pub async fn set_email(code: &str, data: Form<email::Request>) -> Result<(), Status> {
  log::info!("Setting email for code: {}", code);

  let email = data.email.to_owned();

  handle(
    database::update_task_email(code, &email),
    &format!("Updating task email for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;

  Ok(())
}

#[get("/api/gen/<code>")]
pub async fn get_video(code: &str) -> Result<(), Status> {
  log::info!("Get video for code: {}", code);

  let status = handle(
    database::get_task_status(code),
    &format!("Getting task status for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;
  log::debug!("Video status={}", status.to_string());

  match status {
    task::Status::Fail => Err(Status::InternalServerError),
    task::Status::Finish => Ok(()),
    task::Status::Processing => Err(Status::new(499)),
  }
}

#[get("/download/<code>")]
pub async fn download(code: &str) -> Result<rocket::fs::NamedFile, Status> {
  log::info!("Download file for code: {}", code);

  if let Ok(file_path) = get_file_path(code, RESULT_WITH_SUBS_FILE) {
    log::info!("Result with subtitles file found for code: {}", code);

    return handle(
      NamedFile::open(file_path).await,
      &format!("Opening result with subtitles file for code: {}", code),
    )
    .map_err(|_| Status::InternalServerError);
  }

  if let Ok(file_path) = get_file_path(code, RESULT_FILE) {
    log::info!("Result file found for code: {}", code);

    return handle(
      NamedFile::open(file_path).await,
      &format!("Opening result file for code: {}", code),
    )
    .map_err(|_| Status::InternalServerError);
  } else {
    log::warn!("File not found for code: {}", code);
    Err(Status::NotFound)
  }
}

#[get("/file/<code>/<filename>")]
pub fn get_file_path_for_code(code: &str, filename: &str) -> Result<String, Status> {
  get_file_path(code, filename).map_err(|_| Status::NotFound)
}
