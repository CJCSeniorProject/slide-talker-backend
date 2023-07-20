use crate::utils::{create_dir, create_file, get_file_path, make_request};
use std::collections::HashMap;

use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};

pub async fn mp4_to_wav(code: &str) -> Result<(), String> {
  log::info!("Converting MP4 to WAV for code: {}", code);

  let mut map = HashMap::new();
  map.insert("mp4_path", get_file_path(code, "video.mp4")?);
  map.insert("wav_path", create_file(code, "audio.wav")?);

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
  map.insert("audio_path", get_file_path(code, "audio.wav")?);
  map.insert("image_path", get_file_path(code, "avatar.jpg")?);
  map.insert("result_dir", create_dir(code, "gen")?);

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
  map.insert("chunks_dir", get_file_path(code, "gen")?);
  map.insert("output_path", create_file(code, "avatar_video.mp4")?);

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
  map.insert("main_video_path", get_file_path(code, "video.mp4")?);
  map.insert(
    "avatar_video_path",
    get_file_path(code, "avatar_video.mp4")?,
  );
  map.insert("output_path", create_file(code, "result.mp4")?);
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
  map.insert("file_path", get_file_path(code, "audio.wav")?);
  map.insert("save_path", create_file(code, "output.srt")?);

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
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert("video_path", get_file_path(code, "result.mp4")?);
  map.insert("subtitle_path", get_file_path(code, "output.srt")?);
  map.insert("output_path", create_file(code, "result_subtitle.mp4")?);

  let response = make_request("http://localhost:5000/merge_video_and_subtitle", &map).await?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    let err_msg = "Python gen subtitle failed".to_string();
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
