use crate::{
  database,
  model::{constant::*, email, task, video, worker},
  utils::{self, create_dir, create_file},
};
use rocket::{form::Form, get, http::Status, post, serde::json::Json};

#[post("/api/gen", data = "<data>")]
pub async fn gen_video(
  tx: &rocket::State<tokio::sync::mpsc::Sender<worker::Request>>,
  mut data: Form<video::Request<'_>>,
) -> Result<Json<video::Response>, Status> {
  log::info!("Generating video");
  // gen random code
  let code = utils::generate_rand_code();
  log::debug!("Generated code : {}", code);

  create_dir(&code, "").map_err(|_| Status::InternalServerError)?;
  let video_path = create_file(&code, VIDEO_FILE).map_err(|_| Status::InternalServerError)?;
  let avatar_path = create_file(&code, AVATAR_FILE).map_err(|_| Status::InternalServerError)?;

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

#[get("/test")]
pub fn test_fn() -> Result<Status, Status> {
  Ok(Status::UnprocessableEntity)
}

// /file/465787/audio.wav
#[cfg(test)]
mod tests {
  use super::*;
  use rocket::http::{ContentType, Status};
  use rocket::local::blocking::Client;
  use rocket::routes;
  use serde_urlencoded::to_string;

  #[test]
  fn test_set_email() {
    let rocket = rocket::build().mount("/", routes![set_email]);
    let client = Client::untracked(rocket).expect("Failed to create Rocket client");

    let form_data = email::Request {
      email: String::from("example"),
    };

    // 将表单数据转换为 URL 编码的字符串
    let encoded_data = to_string(&form_data).expect("Failed to encode form data");

    // 将 URL 编码的字符串转换为字节数组
    let bytes: Vec<u8> = encoded_data.into_bytes();

    let response = client
      .post("/api/gen/code")
      .header(ContentType::Form)
      .body(bytes)
      .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
  }

  // use reqwest;
  // use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
  // #[actix_rt::test]
  // #[test]
  // fn test_gen_video() {
  //   let mut headers = HeaderMap::new();
  //   headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
  //   headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));

  //   let form = reqwest::blocking::multipart::Form::new()
  //     .file(
  //       "video",
  //       "/home/lab603/Documents/slide_talker_backend/testdata/testvid.mp4",
  //     )
  //     .expect("")
  //     .file(
  //       "avatar",
  //       "/home/lab603/Documents/slide_talker_backend/testdata/testpic.jpg",
  //     )
  //     .expect("")
  //     .text("x", "0.5")
  //     .text("y", "0.5")
  //     .text("shape", "circle")
  //     .text("subtitle", "false");

  // 将文件 Part 添加到表单数据中
  // form = form.part("photo", part);
  // let client = reqwest::blocking::Client::new();
  // let response = client
  //   .post("http://localhost:3000/api/gen")
  //   .header(CONTENT_TYPE, "multipart/form-data")
  //   .multipart(form)
  //   .send()
  //   .expect("");

  // let response = client
  //   .get("http://localhost:3000/test")
  //   .send()
  //   .expect("msg");

  // assert_eq!(response.status(), Status::UnprocessableEntity.code);
  // }
  // #[test]
  // fn test_gen_video() {
  //   let rocket = rocket::build().mount("/", routes![gen_video]);

  //   let client = reqwest::Client::new();
  //   let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  //   let temp_file_video = TempFile::File {
  //     file_name: Some(&FileName::new("video.mp4")),
  //     content_type: Some(ContentType::MP4),
  //     path: Either::Right(PathBuf::from(
  //       "/home/lab603/Documents/slide_talker_backend/testdata/testvid.mp4",
  //     )),
  //     len: 100,
  //   };

  //   let temp_file_avatar = TempFile::File {
  //     file_name: Some(&FileName::new("avatar.jpg")),
  //     content_type: Some(ContentType::JPEG),
  //     path: Either::Right(PathBuf::from(
  //       "/home/lab603/Documents/slide_talker_backend/testdata/testvid.jpg",
  //     )),
  //     len: 100,
  //   };
  //   let file = File::open("path/to/file.txt").expect("Failed to open file");
  //   // 准备测试数据
  //   let form_data = GenVideoRequestForm {
  //     video: file,
  //     avatar: file,
  //     x: 0.5,
  //     y: 0.5,
  //     shape: "circle",
  //     subtitle: false,
  //   };

  //   let response = client
  //     .post("/api/gen")
  //     .header(ContentType::Form)
  //     .body(form)
  //     .dispatch();

  //   assert_eq!(response.status(), Status::UnprocessableEntity);
  // }
}
