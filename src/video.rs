use crate::utils;

use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, Message,
  SmtpTransport, Transport,
};
use std::{fs, path::Path};

mod model;
use model::Request;
pub mod api;

pub async fn start_worker(mut rx: tokio::sync::mpsc::Receiver<Request>) {
  while let Some(request) = rx.recv().await {
    // path define
    let code = &request.code;
    let dir_path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", code);
    let video_path = format!("{}/video.mp4", &dir_path);
    let avatar_path = format!("{}/avatar.jpg", &dir_path);
    let audio_path = format!("{}/audio.wav", &dir_path);
    let result_path = format!("{}/result.mp4", &dir_path);

    utils::mp4_to_wav(&video_path, &audio_path);

    utils::run_gen_video_python(&audio_path, &avatar_path, &dir_path);

    // merge avatar and video, save to tmp/<code>/result.mp4
    utils::merge_video_and_avatar_video(
      &video_path,
      &format!("{}/gen/avatar##audio.mp4", &dir_path),
      &result_path,
      &request.x,
      &request.y,
      &request.shape,
    );

    // if has email.txt, send email to user
    let email_path = format!("{}/email.txt", &dir_path);

    if Path::new(&email_path).exists() {
      let email = fs::read_to_string(&email_path).expect("Something went wrong reading the file");
      let email: Vec<&str> = email.split("\n").collect();
      let email = email[0];

      let body = format!(
        "Hi, your video is ready, please download it from: http://localhost:8000/download/{}",
        code
      );

      // let from = "jimmyhealer <yahing6066@gmail.com>".parse();
      // let reply_to = "jimmyhealer <yahing6066@gmail.com>".parse();
      // let to = email.parse();

      // let email_builder = match (from, reply_to, to) {
      //   (Ok(from), Ok(reply_to), Ok(to)) => Some(
      //     Message::builder()
      //       .from(from)
      //       .reply_to(reply_to)
      //       .to(to)
      //       .subject("Gen video done!")
      //       .header(ContentType::TEXT_PLAIN)
      //       .body(body)
      //       .expect("我一定不會錯"),
      //   ),
      //   _ => None,
      // };

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
    }

    println!("is OK")
  }
}
