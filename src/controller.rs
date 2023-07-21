use crate::{
  model::constant::*,
  utils::{create_dir, create_file, get_file_path, make_request},
};
use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};
use std::{collections::HashMap, fs};

pub async fn mp4_to_wav(code: &str) -> Result<(), String> {
  log::info!("Converting MP4 to WAV for code: {}", code);

  let mut map = HashMap::new();
  map.insert("mp4_path", get_file_path(code, VIDEO_FILE)?);
  map.insert("wav_path", create_file(code, AUDIO_FILE)?);

  let response = make_request("http://localhost:5000/convert_mp4_to_wav", &map).await?;

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
  map.insert("audio_path", get_file_path(code, AUDIO_FILE)?);
  map.insert("image_path", get_file_path(code, AVATAR_FILE)?);
  map.insert("result_dir", create_dir(code, GEN_DIR)?);

  let response = make_request("http://localhost:5000/gen", &map).await?;

  if response.status().is_success() {
    log::info!("Python gen video success");
    Ok(())
  } else {
    let err_msg = "Python gen video failed".to_string();
    log::error!("{}", err_msg);
    Err(err_msg)
  }
}

pub async fn merge_avatar_video_chunks(code: &str) -> Result<(), String> {
  log::info!("Merging video chunks for code: {}", code);

  let mut map = HashMap::new();
  map.insert("chunks_dir", get_file_path(code, GEN_DIR)?);
  map.insert("output_path", create_file(code, AVATAR_VIDEO_FILE)?);

  let response = make_request("http://localhost:5000/merge_avatar_video_chunks", &map).await?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    let err_msg = "FFmpeg merge avatar and video failed".to_string();
    log::error!("{}", err_msg);
    Err(err_msg)
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
  map.insert("main_video_path", get_file_path(code, VIDEO_FILE)?);
  map.insert("avatar_video_path", get_file_path(code, AVATAR_VIDEO_FILE)?);
  map.insert("output_path", create_file(code, RESULT_FILE)?);
  map.insert("position", format!("({},{})", x, y));
  map.insert("avatar_shape", shape.to_string());

  let response = make_request("http://localhost:5000/merge_video_and_avatar_video", &map).await?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    let err_msg = "FFmpeg merge avatar and video failed".to_string();
    log::error!("{}", err_msg);
    Err(err_msg)
  }
}

pub async fn gen_subtitle(code: &str) -> Result<(), String> {
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert("file_path", get_file_path(code, AUDIO_FILE)?);
  map.insert("save_path", create_file(code, SUBS_FILE)?);

  let response = make_request("http://localhost:5000/gen_subtitle", &map).await?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    let err_msg = "Python gen subtitle failed".to_string();
    log::error!("{}", err_msg);
    Err(err_msg)
  }
}

pub async fn merge_video_and_subtitle(code: &str) -> Result<(), String> {
  log::info!("Merging video and subtitle subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert("video_path", get_file_path(code, RESULT_FILE)?);
  map.insert("subtitle_path", get_file_path(code, SUBS_FILE)?);
  map.insert("output_path", create_file(code, RESULT_WITH_SUBS_FILE)?);

  let response = make_request("http://localhost:5000/merge_video_and_subtitle", &map).await?;

  if response.status().is_success() {
    log::info!("Python merge video and subtitle subtitle success");
    Ok(())
  } else {
    let err_msg = "Python merge video and subtitle subtitle failed".to_string();
    log::error!("{}", err_msg);
    Err(err_msg)
  }
}

pub fn send_email(email: &str, code: &str, success: bool) -> Result<(), String> {
  log::info!("Sending email");

  let body = match success {
    true => format!(
      "Hi, your video is ready, please download it from: http://localhost:3000/{}",
      code
    ),
    false => String::from("Video generation failed."),
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

pub fn delete_file_in_dir(code: &str) -> Result<(), String> {
  log::info!("Deleting files in directory for code: {}", code);

  if let Ok(gen) = get_file_path(code, "gen") {
    fs::remove_dir_all(&gen).map_err(|e| {
      let err_msg = format!("Failed to remove directory '{}': {}", gen, e);
      log::error!("{}", err_msg);
      err_msg
    })?;
  };
  let files_to_keep = [
    RESULT_FILE,
    VIDEO_FILE,
    AVATAR_FILE,
    RESULT_WITH_SUBS_FILE,
    SUBS_FILE,
  ];
  let folder_path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", code);
  let dir = fs::read_dir(&folder_path).map_err(|e| {
    let err_msg = format!("Failed to read directory '{}': {}", folder_path, e);
    log::error!("{}", err_msg);
    err_msg
  })?;
  for entry in dir {
    let entry = entry.map_err(|e| {
      let err_msg = format!("Failed to read entry in directory '{}': {}", folder_path, e);
      log::error!("{}", err_msg);
      err_msg
    })?;

    let path = entry.path();

    let file_name = match path.file_name() {
      Some(name) => name.to_string_lossy().to_string(),
      None => continue,
    };

    log::debug!("file={}", file_name);
    if !files_to_keep.contains(&file_name.as_str()) {
      log::debug!("remove file={}", file_name);

      if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| {
          let err_msg = format!("Failed to remove directory '{}': {}", path.display(), e);
          log::error!("{}", err_msg);
          err_msg
        })?;
      } else {
        fs::remove_file(&path).map_err(|e| {
          let err_msg = format!("Failed to remove file '{}': {}", path.display(), e);
          log::error!("{}", err_msg);
          err_msg
        })?;
      }
    }
  }
  log::info!(
    "Deletion of files in directory for code: {} completed",
    code
  );
  Ok(())
}
