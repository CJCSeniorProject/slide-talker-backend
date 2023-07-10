use ffmpeg_next as ffmpeg;
use rand::Rng;

use std::{
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};
use std::{fs, path::Path};

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

pub fn mp4_to_wav(mp4: &str, output_path: &str) -> Result<(), String> {
  log::info!("Converting MP4 to WAV");
  log::debug!("mp4={}, output path={}", mp4, output_path);

  let mut command = Command::new("ffmpeg");
  command
    .arg("-i")
    .arg(mp4)
    .arg("-vn")
    .arg("-acodec")
    .arg("pcm_s16le")
    .arg("-ar")
    .arg("44100")
    .arg("-ac")
    .arg("2")
    .arg("-f")
    .arg("wav")
    .arg(output_path);

  let status = command.status().map_err(|e| {
    log::error!("Failed to execute command: {}", e);
    e.to_string()
  })?;

  if status.success() {
    log::info!("MP4 to WAV conversion success");
    Ok(())
  } else {
    let error_message = format!("MP4 to WAV conversion failed: {:?}", command);
    log::error!("{}", error_message);
    Err(error_message)
  }
}

pub fn run_gen_video_python(wav: &str, avatar: &str, output: &str) -> Result<(), String> {
  log::info!("Running gen video Python script");
  log::debug!(
    "driven_audio={}, source_image={}, result_dir={}",
    wav,
    avatar,
    output
  );

  let mut command = Command::new("zsh");
  command
    .arg("-c")
    .arg(format!("source ~/.zshrc && conda activate sadtalker && python3 ~/Documents/SadTalker/inference.py --driven_audio {} --source_image {} --result_dir {}", wav, avatar, output));

  let status = command.status().map_err(|e| {
    log::error!("Failed to execute command: {}", e);
    e.to_string()
  })?;

  if status.success() {
    log::info!("Python gen video success");
    Ok(())
  } else {
    let error_message = format!("Python gen video failed: {:?}", command);
    log::error!("{}", error_message);
    Err(error_message)
  }
}

fn get_video_decoder(
  file_path: &str,
) -> Result<ffmpeg::codec::decoder::Video, Box<dyn std::error::Error>> {
  log::info!("Getting video decoder for file");
  log::debug!("file={}", file_path);

  ffmpeg::init().map_err(|e| {
    log::error!("Failed to initialize FFmpeg: {}", e);
    e
  })?;

  let ictx = match ffmpeg::format::input(&file_path) {
    Ok(ictx) => ictx,
    Err(e) => {
      log::error!("Failed to open input file: {}", e);
      return Err(e.into());
    }
  };

  let input = match ictx
    .streams()
    .best(ffmpeg::media::Type::Video)
    .ok_or(ffmpeg::Error::StreamNotFound)
  {
    Ok(input) => input,
    Err(e) => {
      log::error!("Failed to find video stream: {}", e);
      return Err(e.into());
    }
  };

  let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())
    .map_err(|e| {
      log::error!("Failed to create decoder context: {}", e);
      e
    })?;

  let decoder = context_decoder.decoder().video().map_err(|e| {
    log::error!("Failed to create video decoder: {}", e);
    e
  })?;

  log::info!("Video decoder obtained successfully");
  Ok(decoder)
}

pub fn merge_video_and_avatar_video(
  video_path: &String,
  avatar_video_path: &String,
  result_path: &String,
  x: &f32,
  y: &f32,
  shape: &String,
) -> Result<(), String> {
  log::info!("Merging video and avatar video");

  let video_decoder = get_video_decoder(&video_path).map_err(|e| {
    log::error!("get_video_decoder error!");
    e.to_string()
  })?;

  let video_size = (video_decoder.width(), video_decoder.height());
  // let avatar_video_size = (avatar_video_decoder.width(), avatar_video_decoder.height());
  let avatar_actual_x = x * video_size.0 as f32;
  let avatar_actual_y = y * video_size.1 as f32;

  let overlay_filter = format!("overlay=x={}:y={}", avatar_actual_x, avatar_actual_y);
  let circle_filter = if shape == "circle" {
    ";[2]alphaextract[vAlpha];[1][vAlpha]alphamerge[bg];[0t25][bg]"
  } else {
    ";[0t25][1]"
  };
  //[1]scale=x=100:y=100[avatar]

  log::debug!(
    "video_size={:?}, avatar_actual_x={}, avatar_actual_y={}, circle_filter={}",
    video_size,
    avatar_actual_x,
    avatar_actual_y,
    circle_filter
  );

  let mut command = Command::new("ffmpeg");
  command
    .arg("-i")
    .arg(&video_path)
    .arg("-i")
    .arg(&avatar_video_path)
    .arg("-i")
    .arg("/home/lab603/Documents/slide_talker_backend/src/circle.png")
    .arg("-filter_complex")
    .arg(format!(
      "[0]fps=25[0t25]{}{}",
      circle_filter, overlay_filter
    ))
    .arg(&result_path);

  let status = command.status().map_err(|e| {
    log::error!("Failed to execute command: {}", e);
    e.to_string()
  })?;

  if status.success() {
    log::info!("FFmpeg merge avatar and video success");
    Ok(())
  } else {
    let error_message = format!("FFmpeg merge avatar and video failed: {:?}", command);
    log::error!("{}", error_message);
    Err(error_message)
  }
}

pub fn send_email(dir_path: &String, code: &String, failed: bool) -> Result<(), String> {
  log::info!("Sending email");

  let email_path = format!("{}/email.txt", &dir_path);
  log::debug!("email_path={}", email_path);

  if Path::new(&email_path).exists() {
    let email = fs::read_to_string(&email_path).expect("Something went wrong reading the file");
    let email: Vec<&str> = email.split("\n").collect();
    let email = email[0];

    let body = match failed {
      false => format!(
        "Hi, your video is ready, please download it from: http://localhost:3000/{}",
        code
      ),
      true => String::from("Your video generation has failed"),
    };

    let email_builder = Message::builder()
      .from("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
      .reply_to("jimmyhealer <yahing6066@gmail.com>".parse().unwrap())
      .to(email.parse().unwrap())
      .subject("Gen video done!")
      .header(ContentType::TEXT_PLAIN)
      .body(body)
      .map_err(|e| {
        log::error!("Failed to build email message: {}", e);
        e.to_string()
      })?;

    let creds = Credentials::new(
      "yahing6066@gmail.com".to_owned(),
      "bdfecnvwtvjksjco".to_owned(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
      .unwrap()
      .credentials(creds)
      .build();

    match mailer.send(&email_builder) {
      Ok(_) => {
        log::info!("Email sent successfully!");
        Ok(())
      }
      Err(e) => {
        log::error!("Could not send email: {:?}", e);
        Err(e.to_string())
      }?,
    }
  } else {
    log::warn!("No email found");
    Ok(())
  }
}

pub fn gen_subtitle(
  dir_path: &String,
  video_path: &String,
  video_subtitle_path: &String,
) -> Result<(), String> {
  log::info!("Generating subtitle");
  log::debug!(
    "dir_path={}, video_path={}, video_subtitle_path={}",
    dir_path,
    video_path,
    video_subtitle_path
  );

  let mut command = Command::new("zsh");
  command
    .arg("-c")
    .arg(format!("source ~/.zshrc && conda activate sadtalker && python3 ~/Documents/SadTalker/ai_subtitle.py --file {} --save_dir {}", video_path, dir_path));

  let status = command.status().map_err(|e| {
    log::error!("Failed to execute command: {}", e);
    e.to_string()
  })?;

  if status.success() {
    log::info!("Python gen subtitle success");
  } else {
    let error_message = format!("Python gen subtitle failed: {:?}", command);
    log::error!("{}", error_message);
    return Err(error_message);
  }

  let mut command = Command::new("ffmpeg");
  command
    .arg("-i")
    .arg(&video_path)
    .arg("-vf")
    .arg(format!("subtitles={}/output_crt.srt", dir_path))
    .arg("-y")
    .arg(&video_subtitle_path);

  let status = command.status().map_err(|e| {
    log::error!("Failed to execute command: {}", e);
    e.to_string()
  })?;

  if status.success() {
    log::info!("FFmpeg composite subtitle success");
    Ok(())
  } else {
    let error_message = format!("FFmpeg composite subtitle failed: {:?}", command);
    log::error!("{}", error_message);
    return Err(error_message);
  }
}
