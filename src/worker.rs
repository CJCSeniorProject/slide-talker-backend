use crate::controller::{
  gen_subtitle, merge_avatar_video_chunks, merge_video_and_avatar_video, merge_video_and_subtitle,
  mp4_to_wav, run_gen_video_python, send_email,
};
use crate::database;
use crate::model::{task, worker};
use crate::utils::*;
use tokio::sync::mpsc::Receiver;

pub async fn start_worker(mut rx: Receiver<worker::Request>) {
  log::info!("Starting video generation worker!");
  while let Some(request) = rx.recv().await {
    log::info!("Received request with code: {}", request.code);
    let code = &request.code;

    if let Err(_) = mp4_to_wav(code).await {
      result(code, false);
      continue;
    }

    if let Err(_) = run_gen_video_python(code).await {
      result(code, false);
      continue;
    }

    if let Err(_) = merge_avatar_video_chunks(code).await {
      result(code, false);
      continue;
    }

    // merge avatar and video, save to tmp/<code>/result.mp4
    if let Err(_) = merge_video_and_avatar_video(code, &request.x, &request.y, &request.shape).await
    {
      result(code, false);
      continue;
    }

    // if need generate subtitle
    if request.subtitle {
      if let Err(_) = gen_subtitle(code).await {
        result(code, false);
        continue;
      }

      if let Err(_) = merge_video_and_subtitle(code).await {
        result(code, false);
        continue;
      }
    }

    result(code, true);
    log::info!("Video generation completed for code: {}", code);
  }
}

// check worker result and update database status
fn result(code: &str, success: bool) {
  log::info!("Result of code: {}, success={}", code, success);
  if success {
    let _ = database::update_task_status(code, task::Status::Finish);
  } else {
    let _ = database::update_task_status(code, task::Status::Fail);
  }
  match database::get_task_email(code) {
    Ok(email) => {
      let _ = send_email(&email, code, success);
    }
    Err(_) => {
      log::warn!("Email not found");
    }
  }
  let _ = delete_file_in_dir(code);
}
