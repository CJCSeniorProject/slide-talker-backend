use crate::{
  database,
  model::{
    constant::*,
    task::Status::{Fail, Finish, Processing},
    *,
  },
  utils::*,
};
use rocket::{form::Form, fs::NamedFile, get, http::Status, post, serde::json::Json, State};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[post("/api/gen", data = "<data>")]
pub async fn gen_video(
  sender: &State<worker::Sender>,
  mut data: Form<video::Request<'_>>,
) -> Result<Json<Value>, Status> {
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
  log::debug!("data={:?}", data);

  // 新增任務至資料庫
  handle(
    database::insert_task(&code, data.subtitle),
    &format!("Inserting task for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;

  // send request to gen worker
  let request = worker::GenVideoRequest {
    code: code.clone(),
    x: data.x,
    y: data.y,
    shape: data.shape.clone(),
    remove_bg: data.remove_bg,
    subtitle: data.subtitle,
  };
  log::debug!("request={:?}", request);

  let tx = &sender.gen_sender;
  match tx.try_send(request) {
    Ok(_) => {
      log::info!("Video generation request sent for code: {}", code);
    }
    Err(e) => match e {
      mpsc::error::TrySendError::Full(_) => Err(Status::ServiceUnavailable)?,
      mpsc::error::TrySendError::Closed(_) => Err(Status::InternalServerError)?,
    },
  }

  let response = json!({
    "code": code
  });
  Ok(Json(response))
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
pub async fn check_task_status(code: &str) -> Result<(), Status> {
  log::info!("Checking task status for code: {}", code);

  // TODO
  if code == "undefined" {
    log::warn!("code: undefined");
    return Err(Status::new(498));
  }
  let task = handle(
    database::get_task_info(code),
    &format!("Getting task info for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;
  log::debug!("task={:?}", task);

  match task.status {
    Fail => Err(Status::InternalServerError),
    Finish => Ok(()),
    Processing => Err(Status::new(499)),
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

#[get("/api/gen/subtitle/<code>")]
pub async fn gen_subtitle(code: &str) -> Result<(), Status> {
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert(
    "file_path",
    handle(get_file_path(code, AUDIO_FILE), "Inserting file_path")
      .map_err(|_| Status::InternalServerError)?,
  );
  map.insert(
    "save_path",
    handle(create_file(code, SUBS_FILE), "Inserting save_path")
      .map_err(|_| Status::InternalServerError)?,
  );

  let response = handle(
    make_request("http://localhost:5000/gen_subtitle", &map).await,
    "Making request",
  )
  .map_err(|_| Status::InternalServerError)?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    Err(Status::InternalServerError)
  }
}

#[post("/api/set/subtitle/<code>", data = "<data>")]
pub async fn set_subtitle(
  sender: &State<worker::Sender>,
  code: &str,
  data: Form<subtitle::Request>,
) -> Result<(), Status> {
  log::info!("Setting subtitle for code: {}", code);

  let subs = &data.subtitles;

  handle(
    database::update_task_subtitles(code, subs),
    &format!("Updating task subtitles for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;

  handle(
    database::update_subtitles_status(code, Finish),
    &format!("Updating subtitles status for code: {}", code),
  )
  .map_err(|_| Status::InternalServerError)?;

  // send request to merge worker
  let request = worker::MergeSubsRequest {
    code: code.to_string(),
  };

  let tx = &sender.merge_sender;
  handle(tx.try_send(request), "Sending request to merge worker")
    .map_err(|_| Status::ServiceUnavailable)?;

  log::info!("Merge request sent for code: {}", code);

  Ok(())
}

#[get("/file/<code>/<filename>")]
pub fn get_file_path_for_code(code: &str, filename: &str) -> Result<String, Status> {
  get_file_path(code, filename).map_err(|_| Status::NotFound)
}

// #[tokio::test]
// async fn test_subs() {
//   let sub1 = subtitle::Subtitle {
//     text: "sub1".to_string(),
//     fontsize: 32,
//     color: "white".to_string(),
//     font: "./NotoSansCJK-Regular.ttc".to_string(),
//     start_time: "00:00:00,000".to_string(),
//     end_time: "00:00:00,500".to_string(),
//   };
//   let sub2 = subtitle::Subtitle {
//     text: "sub2".to_string(),
//     fontsize: 32,
//     color: "white".to_string(),
//     font: "./NotoSansCJK-Regular.ttc".to_string(),
//     start_time: "00:00:00,500".to_string(),
//     end_time: "00:00:01,000".to_string(),
//   };
//   let subtitles = vec![sub1, sub2];

//   let mut data = HashMap::new();
//   data.insert(
//     "subtitles",
//     serde_json::to_value(subtitles).expect("to_value err"),
//   );
//   data.insert("video_path", Value::String("testvideo".to_string()));
//   data.insert("output_path", Value::String("testoutput".to_string()));

//   let response = handle(
//     make_request("http://localhost:5000/set_subtitle", &data).await,
//     "Making request",
//   )
//   .map_err(|_| Status::InternalServerError);

//   assert!(response.is_ok());
// }
