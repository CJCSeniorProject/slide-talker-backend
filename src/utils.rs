use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};
use rand::Rng;
use reqwest;
use std::{
  collections::HashMap,
  time::{SystemTime, UNIX_EPOCH},
};

use crate::db;
use crate::video::{api::get_file_path, model::TaskStatus};

pub fn generate_rand_code() -> String {
  log::info!("Generating random code");

  let mut rng = rand::thread_rng();
  let charset: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let rand_num: u128 = rng.gen();
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let offset: usize = match (rand_num % 93229 * timestamp % 93229).try_into() {
    Ok(offset) => {
      log::debug!("Offset calculated: {}", offset);
      offset
    }
    Err(_) => {
      log::warn!("Failed to convert offset, using default value 0");
      0
    }
  };

  let random_code: String = (0..5)
    .map(|_| {
      let idx = (rng.gen_range(0..charset.len()) + offset) % charset.len();
      charset[idx] as char
    })
    .collect();

  log::debug!(
    "random_code={}, timestamp={}",
    random_code,
    timestamp % 1000
  );

  let code: String = format!("{}{}", random_code, timestamp % 1000);
  log::info!("Random code generated, code = {}", code);
  code
}

pub async fn mp4_to_wav(code: &str) -> Result<(), String> {
  log::info!("Converting MP4 to WAV for code: {}", code);

  let mut map = HashMap::new();
  map.insert("mp4_path", get_file_path(code, "video.mp4"));
  map.insert("wav_path", get_file_path(code, "audio.wav"));

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/convert_mp4_to_wav")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("MP4 to WAV conversion success");
    Ok(())
  } else {
    let err_msg = "MP4 to WAV conversion failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub async fn run_gen_video_python(code: &str) -> Result<(), String> {
  log::info!("Running gen video Python script for code: {}", code);

  let mut map = HashMap::new();
  map.insert("audio_path", get_file_path(code, "audio.wav"));
  map.insert("image_path", get_file_path(code, "avatar.jpg"));
  map.insert("result_dir", get_file_path(code, "gen"));

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/gen")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("Python gen video success");
    Ok(())
  } else {
    let err_msg = "Python gen video failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub async fn merge_avatar_video_chunks(code: &str) -> Result<(), String> {
  log::info!("Merging video chunks for code: {}", code);

  let mut map = HashMap::new();
  map.insert("chunks_dir", get_file_path(code, "gen"));
  map.insert("output_path", get_file_path(code, "avatar_video.mp4"));

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/merge_avatar_video_chunks")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    let err_msg = "FFmpeg merge avatar and video failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub async fn merge_video_and_avatar_video(
  code: &str,
  x: &f32,
  y: &f32,
  shape: &str,
) -> Result<(), String> {
  log::info!("Merging video and avatar video for code: {}", code);

  let mut map = HashMap::new();
  map.insert("main_video_path", get_file_path(code, "video.mp4"));
  map.insert("avatar_video_path", get_file_path(code, "avatar_video.mp4"));
  map.insert("output_path", get_file_path(code, "result.mp4"));
  map.insert("position", format!("({},{})", x, y));
  map.insert("avatar_shape", shape.to_string());

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/merge_video_and_avatar_video")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    let err_msg = "FFmpeg merge avatar and video failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub async fn gen_subtitle(code: &str) -> Result<(), String> {
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert("file_path", get_file_path(code, "audio.wav"));
  map.insert("save_path", get_file_path(code, "output.srt"));

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/gen_subtitle")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    let err_msg = "Python gen subtitle failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub async fn merge_video_and_subtitle(code: &str) -> Result<(), String> {
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert("video_path", get_file_path(code, "result.mp4"));
  map.insert("subtitle_path", get_file_path(code, "output.srt"));
  map.insert("output_path", get_file_path(code, "result_subtitle.mp4"));

  let client = reqwest::Client::new();
  let response = client
    .post("http://localhost:5000/merge_video_and_subtitle")
    .json(&map)
    .send()
    .await
    .map_err(|e| {
      let err_msg = format!("Request failed with error: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    let err_msg = "Python gen subtitle failed";
    log::error!("{}", err_msg);
    Err(err_msg.to_string())
  }
}

pub fn result(code: &str, success: bool) {
  log::info!("Result of code: {}, success={}", code, success);
  if success {
    let _ = db::update_task_status(code, TaskStatus::Finish);
  } else {
    let _ = db::update_task_status(code, TaskStatus::Fail);
  }
  match db::get_task_email(code) {
    Ok(email) => {
      let _ = send_email(&email, code, success);
    }
    Err(_) => {
      log::warn!("Email not found");
    }
  }
}

pub fn send_email(email: &str, code: &str, success: bool) -> Result<(), String> {
  log::info!("Sending email");

  let body = match success {
    true => format!(
      "Hi, your video is ready, please download it from: http://localhost:3000/{}",
      code
    ),
    false => String::from("Your video generation has failed"),
  };

  let creds = Credentials::new(
    "yahing6066@gmail.com".to_owned(),
    "bdfecnvwtvjksjco".to_owned(),
  );

  // Open a remote connection to gmail
  let mailer = SmtpTransport::relay("smtp.gmail.com")
    .unwrap()
    .credentials(creds)
    .build();

  let email_builder = Message::builder()
    .from("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
    .reply_to("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
    .to(email.parse().unwrap())
    .subject("Gen video done!")
    .header(ContentType::TEXT_PLAIN)
    .body(body)
    .map_err(|e| {
      let err_msg = format!("Failed to build email message: {:?}", e);
      log::error!("{}", err_msg);
      err_msg
    })?;

  match mailer.send(&email_builder) {
    Ok(_) => {
      log::info!("Email sent successfully!");
      Ok(())
    }
    Err(e) => {
      let err_msg = format!("Failed to send email: {:?}", e);
      log::error!("{}", err_msg);
      Err(err_msg)
    }
  }
}
