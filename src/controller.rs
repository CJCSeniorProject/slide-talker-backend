use crate::{model::constant::*, utils::*};
use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};
use std::collections::HashMap;

pub async fn mp4_to_wav(code: &str) -> Result<(), Error> {
  log::info!("Converting MP4 to WAV for code: {}", code);

  let mut map = HashMap::new();
  map.insert(
    "mp4_path",
    handle(get_file_path(code, VIDEO_FILE), "Inserting mp4_path")?,
  );
  map.insert(
    "wav_path",
    handle(create_file(code, AUDIO_FILE), "Inserting wav_path")?,
  );

  let response = handle(
    make_request("http://localhost:5000/convert_mp4_to_wav", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("MP4 to WAV conversion success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub async fn run_gen_video_python(code: &str) -> Result<(), Error> {
  log::info!("Running gen video Python script for code: {}", code);

  let mut map = HashMap::new();
  map.insert(
    "audio_path",
    handle(get_file_path(code, AUDIO_FILE), "Inserting audio_path")?,
  );
  map.insert(
    "image_path",
    handle(get_file_path(code, AVATAR_FILE), "Inserting image_path")?,
  );
  map.insert(
    "result_dir",
    handle(create_dir(code, GEN_DIR), "Inserting result_dir")?,
  );

  let response = handle(
    make_request("http://localhost:5000/gen", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("Python gen video success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub async fn merge_avatar_video_chunks(code: &str) -> Result<(), Error> {
  log::info!("Merging video chunks for code: {}", code);

  let mut map = HashMap::new();
  map.insert(
    "chunks_dir",
    handle(get_file_path(code, GEN_DIR), "Inserting chunks_dir")?,
  );
  map.insert(
    "output_path",
    handle(
      create_file(code, AVATAR_VIDEO_FILE),
      "Inserting output_path",
    )?,
  );

  let response = handle(
    make_request("http://localhost:5000/merge_avatar_video_chunks", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub async fn merge_video_and_avatar_video(
  code: &str,
  x: &f32,
  y: &f32,
  shape: &str,
) -> Result<(), Error> {
  log::info!("Merging video and avatar video for code: {}", code);

  let mut map = HashMap::new();
  map.insert(
    "main_video_path",
    handle(get_file_path(code, VIDEO_FILE), "Inserting main_video_path")?,
  );
  map.insert(
    "avatar_video_path",
    handle(
      get_file_path(code, AVATAR_VIDEO_FILE),
      "Inserting avatar_video_path",
    )?,
  );
  map.insert(
    "output_path",
    handle(create_file(code, RESULT_FILE), "Inserting output_path")?,
  );
  map.insert("position", format!("({},{})", x, y));
  map.insert("avatar_shape", shape.to_string());

  let response = handle(
    make_request("http://localhost:5000/merge_video_and_avatar_video", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub async fn gen_subtitle(code: &str) -> Result<(), Error> {
  log::info!("Generating subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert(
    "file_path",
    handle(get_file_path(code, AUDIO_FILE), "Inserting file_path")?,
  );
  map.insert(
    "save_path",
    handle(create_file(code, SUBS_FILE), "Inserting save_path")?,
  );

  let response = handle(
    make_request("http://localhost:5000/gen_subtitle", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("Python gen subtitle success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub async fn merge_video_and_subtitle(code: &str) -> Result<(), Error> {
  log::info!("Merging video and subtitle subtitle for code: {}", &code);

  let mut map = HashMap::new();
  map.insert(
    "video_path",
    handle(get_file_path(code, RESULT_FILE), "Inserting video_path")?,
  );
  map.insert(
    "subtitle_path",
    handle(get_file_path(code, SUBS_FILE), "Inserting subtitle_path")?,
  );
  map.insert(
    "output_path",
    handle(
      create_file(code, RESULT_WITH_SUBS_FILE),
      "Inserting output_path",
    )?,
  );

  let response = handle(
    make_request("http://localhost:5000/merge_video_and_subtitle", &map).await,
    "Making request",
  )?;

  if response.status().is_success() {
    log::info!("Python merge video and subtitle subtitle success");
    Ok(())
  } else {
    Err(Error::new(ErrorKind::Other, ""))
  }
}

pub fn send_email(email: &str, code: &str, success: bool) -> Result<(), Error> {
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

  let email_builder = handle(
    Message::builder()
      .from("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
      .reply_to("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
      .to(email.parse().unwrap())
      .subject("Gen video done!")
      .header(ContentType::TEXT_PLAIN)
      .body(body),
    "Building email message",
  )?;

  handle(mailer.send(&email_builder), "Sending email")?;

  log::info!("Email sent successfully!");
  Ok(())
}
