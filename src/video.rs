use crate::utils;

pub mod api;
pub mod model;
use model::Request;

pub async fn start_worker(mut rx: tokio::sync::mpsc::Receiver<Request>) {
  while let Some(request) = rx.recv().await {
    // path define
    let code = &request.code;
    let dir_path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", code);
    let video_path = format!("{}/video.mp4", &dir_path);
    let avatar_path = format!("{}/avatar.jpg", &dir_path);
    let audio_path = format!("{}/audio.wav", &dir_path);
    let result_path = format!("{}/result.mp4", &dir_path);
    let result_subtitle_path = format!("{}/result_subtitle.mp4", &dir_path);

    if let Err(e) = utils::mp4_to_wav(&video_path, &audio_path) {
      println!("error : {}", e);
      utils::send_email(&dir_path, code, true);
      continue;
    }

    if let Err(e) = utils::run_gen_video_python(&audio_path, &avatar_path, &dir_path) {
      println!("error : {}", e);
      utils::send_email(&dir_path, code, true);
      continue;
    }

    // merge avatar and video, save to tmp/<code>/result.mp4
    if let Err(e) = utils::merge_video_and_avatar_video(
      &video_path,
      &format!("{}/gen/avatar##audio.mp4", &dir_path),
      &result_path,
      &request.x,
      &request.y,
      &request.shape,
    ) {
      println!("error : {}", e);
      utils::send_email(&dir_path, code, true);
      continue;
    }

    // if need generate subtitle
    if request.subtitle {
      if let Err(e) = utils::gen_subtitle(&dir_path, &result_path, &result_subtitle_path) {
        println!("error : {}", e);
        utils::send_email(&dir_path, code, true);
        continue;
      }
    }

    // if has email.txt, send email to user
    utils::send_email(&dir_path, code, false);

    println!("is OK")
  }
}
