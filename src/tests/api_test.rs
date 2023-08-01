use super::common::*;
use crate::api::*;

use dotenv::dotenv;
use rocket::{
  http::{ContentType, Header, Status},
  local::blocking::Client,
  routes,
};
use serde_urlencoded::to_string;
use tokio::sync::mpsc;

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

#[test]
fn test_gen_video() {
  dotenv().ok();
  let (tx, _rx): (
    mpsc::Sender<worker::Request>,
    mpsc::Receiver<worker::Request>,
  ) = mpsc::channel(1);

  let rocket = rocket::build().mount("/", routes![gen_video]).manage(tx);
  let client = Client::untracked(rocket).expect("valid rocket instance");

  let content_type = Header::new(
    "Content-Type",
    format!("multipart/form-data; boundary={}", BOUNDARY),
  );

  let response = client
    .post("/api/gen")
    .header(content_type)
    .body(
      create_form_data(
        BOUNDARY,
        "testvid.mp4",
        "testpic.jpg",
        "0.5",
        "0.5",
        "circle",
        "false",
      )
      .unwrap(),
    )
    .dispatch();

  assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_gen_video_missing_data() {
  dotenv().ok();
  let (tx, _rx): (
    mpsc::Sender<worker::Request>,
    mpsc::Receiver<worker::Request>,
  ) = mpsc::channel(1);

  let rocket = rocket::build().mount("/", routes![gen_video]).manage(tx);
  let client = Client::untracked(rocket).expect("valid rocket instance");

  let content_type = Header::new(
    "Content-Type",
    format!("multipart/form-data; boundary={}", BOUNDARY),
  );

  let response = client
    .post("/api/gen")
    .header(content_type.clone())
    .body(create_form_data(BOUNDARY, "", "testpic.jpg", "0.5", "0.5", "circle", "false").unwrap())
    .dispatch();

  assert_eq!(response.status(), Status::UnprocessableEntity);

  let response = client
    .post("/api/gen")
    .header(content_type)
    .body(create_form_data(BOUNDARY, "testvid.mp4", "", "0.5", "0.5", "circle", "false").unwrap())
    .dispatch();

  assert_eq!(response.status(), Status::UnprocessableEntity);
}

#[test]
fn test_gen_video_invalid_data() {
  dotenv().ok();
  let (tx, _rx): (
    mpsc::Sender<worker::Request>,
    mpsc::Receiver<worker::Request>,
  ) = mpsc::channel(1);

  let rocket = rocket::build().mount("/", routes![gen_video]).manage(tx);
  let client = Client::untracked(rocket).expect("valid rocket instance");

  let content_type = Header::new(
    "Content-Type",
    format!("multipart/form-data; boundary={}", BOUNDARY),
  );

  let response = client
    .post("/api/gen")
    .header(content_type)
    .body(
      create_form_data(
        BOUNDARY,
        "testpic.jpg",
        "testvid.mp4",
        "0.5",
        "0.5",
        "circle",
        "false",
      )
      .unwrap(),
    )
    .dispatch();

  assert_eq!(response.status(), Status::UnprocessableEntity);
}

#[test]
fn test_gen_video_data_out_of_range() {
  dotenv().ok();
  let (tx, _rx): (
    mpsc::Sender<worker::Request>,
    mpsc::Receiver<worker::Request>,
  ) = mpsc::channel(1);

  let rocket = rocket::build().mount("/", routes![gen_video]).manage(tx);
  let client = Client::untracked(rocket).expect("valid rocket instance");

  let content_type = Header::new(
    "Content-Type",
    format!("multipart/form-data; boundary={}", BOUNDARY),
  );

  let response = client
    .post("/api/gen")
    .header(content_type)
    .body(
      create_form_data(
        BOUNDARY,
        "testvid.mp4",
        "testpic.jpg",
        "1.5",
        "-0.5",
        "circle",
        "false",
      )
      .unwrap(),
    )
    .dispatch();

  assert_eq!(response.status(), Status::UnprocessableEntity);
}

#[test]
fn test_get_video_finish() {
  let code = "finish";
  let status = task::Status::Finish;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![get_video]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_get_video_fail() {
  let code = "fail";
  let status = task::Status::Fail;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![get_video]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::InternalServerError);
}

#[test]
fn test_get_video_processing() {
  let code = "process";
  let status = task::Status::Processing;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![get_video]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::new(499));
}
