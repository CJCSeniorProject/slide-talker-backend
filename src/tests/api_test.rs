use super::common::*;
use crate::api::*;
use dotenv::dotenv;
use rocket::{
  form::Form,
  http::{ContentType, Header, Status},
  local::blocking::Client,
  routes,
};
use serde_urlencoded::to_string;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[test]
fn test_gen_video() {
  dotenv().ok();
  let (tx, _rx): (
    mpsc::Sender<worker::GenVideoRequest>,
    mpsc::Receiver<worker::GenVideoRequest>,
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
    mpsc::Sender<worker::GenVideoRequest>,
    mpsc::Receiver<worker::GenVideoRequest>,
  ) = mpsc::channel(1);

  let rocket = rocket::build().mount("/", routes![gen_video]).manage(tx);
  let client = Client::untracked(rocket).expect("valid rocket instance");

  let content_type = Header::new(
    "Content-Type",
    format!("multipart/form-data; boundary={}", BOUNDARY),
  );

  // miss video path
  let response = client
    .post("/api/gen")
    .header(content_type.clone())
    .body(create_form_data(BOUNDARY, "", "testpic.jpg", "0.5", "0.5", "circle", "false").unwrap())
    .dispatch();

  assert_eq!(response.status(), Status::UnprocessableEntity);

  // miss avatar path
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
    mpsc::Sender<worker::GenVideoRequest>,
    mpsc::Receiver<worker::GenVideoRequest>,
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
    mpsc::Sender<worker::GenVideoRequest>,
    mpsc::Receiver<worker::GenVideoRequest>,
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
fn test_check_task_status_finish() {
  let code = "finish";
  let status = task::Status::Finish;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![check_task_status]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_check_task_status_fail() {
  let code = "fail";
  let status = task::Status::Fail;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![check_task_status]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::InternalServerError);
}

#[test]
fn test_check_task_status_processing() {
  let code = "process";
  let status = task::Status::Processing;
  insert_task_with_status(code, status);
  let rocket = rocket::build().mount("/", routes![check_task_status]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let response = client.get(format!("/api/gen/{}", code)).dispatch();

  delete_task_by_code(code);

  assert_eq!(response.status(), Status::new(499));
}

#[test]
fn test_set_email() {
  let rocket = rocket::build().mount("/", routes![set_email]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let mut form_data = HashMap::new();
  form_data.insert("email", "example_email");

  // 将表单数据转换为 URL 编码的字符串
  let encoded_data = to_string(&form_data).expect("Failed to encode form data");
  // 将 URL 编码的字符串转换为字节数组
  let bytes: Vec<u8> = encoded_data.into_bytes();

  let response = client
    .post("/api/gen/code")
    .header(ContentType::Form)
    .body(bytes)
    .dispatch();

  assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_set_subtitle() {
  let rocket = rocket::build().mount("/", routes![set_subtitle]);
  let client = Client::untracked(rocket).expect("Failed to create Rocket client");

  let subtitle1 = subtitle::Subtitle {
    text: "da".to_string(),
    fontsize: 32,
    color: "white".to_string(),
    font: "./NotoSansCJK-Regular.ttc".to_string(),
    start_time: "00:00:00,000".to_string(),
    end_time: "00:00:00,500".to_string(),
  };

  let subtitle2 = subtitle::Subtitle {
    text: "subtitle2".to_string(),
    fontsize: 32,
    color: "white".to_string(),
    font: "./NotoSansCJK-Regular.ttc".to_string(),
    start_time: "00:00:01,000".to_string(),
    end_time: "00:00:01,500".to_string(),
  };

  let mut form_data = HashMap::new();
  form_data.insert(
    "subtitles[0]",
    serde_urlencoded::to_string(subtitle1).unwrap(),
  );
  form_data.insert(
    "subtitles[1]",
    serde_urlencoded::to_string(subtitle2).unwrap(),
  );
  let data = serde_urlencoded::to_string(&form_data).unwrap();

  // let data = form_data
  //   .iter()
  //   .map(|(k, v)| format!("{}={}", k, v))
  //   .collect::<Vec<String>>()
  //   .join("&");

  // println!("{:?}", form_data);
  // let bytes: Vec<u8> = form_data.into_bytes();

  let response = client
    .post("/api/set_subtitle/code")
    .header(ContentType::Form)
    .body(data)
    .dispatch();

  assert_eq!(response.status(), Status::Ok);
}
