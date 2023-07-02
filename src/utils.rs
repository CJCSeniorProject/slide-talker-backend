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
  let mut rng = rand::thread_rng();
  let charset: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let rand_num: u128 = rng.gen();
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let offset: usize = match (rand_num % 93229 * timestamp % 93229).try_into() {
    Ok(offset) => offset,
    Err(_) => 0,
  };

  let code: String = (0..5)
    .map(|_| {
      let idx = (rng.gen_range(0..charset.len()) + offset) % charset.len();
      charset[idx] as char
    })
    .collect();

  let code: String = format!("{}{}", code, timestamp % 1000);
  code
}

pub fn mp4_to_wav(mp4: &str, output_path: &str) -> Result<(), String> {
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

  let status = command.status().expect("failed to execute mp4_to_wav");
  if status.success() {
    Ok(())
  } else {
    Err(String::from(format!(
      "ffmpeg convert mp4 to wav failed: {:?}",
      command
    )))
  }
}

pub fn run_gen_video_python(wav: &str, avatar: &str, output: &str) -> Result<(), String> {
  let mut command = Command::new("zsh");
  command
    .arg("-c")
    .arg(format!("source ~/.zshrc && conda activate sadtalker && python3 ~/Documents/SadTalker/inference.py --driven_audio {} --source_image {} --result_dir {}", wav, avatar, output));

  let status = command
    .status()
    .expect("failed to execute run_gen_video_python");
  if status.success() {
    println!("python gen video success");
    Ok(())
  } else {
    Err(String::from(format!(
      "python gen video failed: {:?}",
      command
    )))
  }
}

fn get_video_decoder(
  file_path: &str,
) -> Result<ffmpeg::codec::decoder::Video, Box<dyn std::error::Error>> {
  ffmpeg::init()?;

  let ictx = match ffmpeg::format::input(&file_path) {
    Ok(ictx) => ictx,
    Err(e) => return Err(e.into()),
  };

  let input = match ictx
    .streams()
    .best(ffmpeg::media::Type::Video)
    .ok_or(ffmpeg::Error::StreamNotFound)
  {
    Ok(input) => input,
    Err(e) => return Err(e.into()),
  };

  let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
  let decoder = context_decoder.decoder().video()?;

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
  let video_decoder = match get_video_decoder(&video_path) {
    Ok(decoder) => decoder,
    Err(e) => return Err(String::from(format!("get video failed: {:?}", e))),
  };

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

  let status = command
    .status()
    .expect("failed to execute merge_video_and_avatar_video");
  if status.success() {
    println!("ffmpeg merge avatar and video success");
    Ok(())
  } else {
    Err(String::from(format!(
      "ffmpeg merge avatar and video failed: {:?}",
      command
    )))
  }
}

pub fn send_email(dir_path: &String, code: &String, failed: bool) {
  let email_path = format!("{}/email.txt", &dir_path);

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
      .unwrap();

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
      Ok(_) => println!("Email sent successfully!"),
      Err(e) => panic!("Could not send email: {e:?}"),
    }
  } else {
    println!("no email");
  }
}

pub fn gen_subtitle(
  dir_path: &String,
  video_path: &String,
  video_subtitle_path: &String,
) -> Result<(), String> {
  let mut command = Command::new("zsh");
  command
    .arg("-c")
    .arg(format!("source ~/.zshrc && conda activate sadtalker && python3 ~/Documents/SadTalker/ai_subtitle.py --file {} --save_dir {}", video_path, dir_path));

  let status = command.status().expect("failed to execute gen_ai_subtitle");
  if status.success() {
    println!("python gen subtitle success");
  } else {
    return Err(String::from(format!(
      "python gen subtitle failed: {:?}",
      command
    )));
  }

  let mut command = Command::new("ffmpeg");
  command
    .arg("-i")
    .arg(&video_path)
    .arg("-vf")
    .arg(format!("subtitles={}/output_crt.srt", dir_path))
    .arg("-y")
    .arg(&video_subtitle_path);

  let status = command
    .status()
    .expect("failed to execute composite_subtitle");

  if status.success() {
    println!("ffmpeg composite subtitle success");
    Ok(())
  } else {
    Err(String::from(format!(
      "ffmpeg composite subtitle failed: {:?}",
      command
    )))
  }
}
