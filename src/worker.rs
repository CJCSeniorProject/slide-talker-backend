use crate::{
  controller::*,
  database,
  model::{task, worker},
  utils::*,
};
use tokio::sync::mpsc::Receiver;

pub async fn start_worker(mut rx: Receiver<worker::Request>) {
  log::info!("Starting video generation worker!");

  while let Some(request) = rx.recv().await {
    log::info!("Received request with code: {}", request.code);

    let code = &request.code;

    if let Err(_) = handle(
      mp4_to_wav(code).await,
      &format!("Running mp4_to_wav for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    if let Err(_) = handle(
      run_gen_video_python(code).await,
      &format!("Running run_gen_video_python for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    if let Err(_) = handle(
      merge_avatar_video_chunks(code).await,
      &format!("Running mp4_to_wav for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // merge avatar and video, save to tmp/<code>/result.mp4
    if let Err(_) = handle(
      merge_video_and_avatar_video(code, &request.x, &request.y, &request.shape).await,
      &format!("Running mp4_to_wav for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // if need generate subtitle
    if request.subtitle {
      if let Err(_) = handle(
        gen_subtitle(code).await,
        &format!("Running mp4_to_wav for code: {}", code),
      ) {
        let _ = result(code, false);
        continue;
      }

      if let Err(_) = handle(
        merge_video_and_subtitle(code).await,
        &format!("Running mp4_to_wav for code: {}", code),
      ) {
        let _ = result(code, false);
        continue;
      }
    }

    let _ = result(code, true);
    log::info!("Video generation completed for code: {}", code);
  }
}

// check worker result and update database status
fn result(code: &str, success: bool) -> Result<(), Error> {
  log::info!("Result of code: {}, success={}", code, success);

  if success {
    handle(
      database::update_task_status(code, task::Status::Finish),
      &format!("Updating task status 'Finish' for code: {}", code),
    )?;
  } else {
    handle(
      database::update_task_status(code, task::Status::Fail),
      &format!("Updating task status 'Fail' for code: {}", code),
    )?;
  }
  match handle(
    database::get_task_email(code),
    &format!("Getting task email with code: {}", code),
  )? {
    Some(email) => {
      handle(
        send_email(&email, code, success),
        &format!("Sending email for code: {}", code),
      )?;
    }
    None => {
      log::warn!("Email not found for code: {}", code);
    }
  }
  handle(
    delete_file_in_dir(code),
    &format!("Deleting file in directoey '{}'", code),
  )?;
  Ok(())
}
