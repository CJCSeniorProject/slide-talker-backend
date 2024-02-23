use crate::{
  controller::*,
  database,
  model::{
    constant::*,
    task::{
      Status::{Fail, Finish},
      Task,
    },
    worker,
  },
  utils::*,
};
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn start_gen_video_worker(
  mut rx: Receiver<worker::GenVideoRequest>,
  tx: Sender<worker::MergeSubsRequest>,
) {
  log::info!("Starting video generation worker!");

  while let Some(request) = rx.recv().await {
    log::info!("Received a request to gen video for code: {}", request.code);

    let code = &request.code;
    let x = request.x;
    let y = request.y;
    let shape = request.shape;
    let subtitle = request.subtitle;
    let remove_bg = request.remove_bg;

    // 移除背景
    if remove_bg {
      if let Err(_) = handle(
        remove_background(code).await,
        &format!("Removing background for code: {}", code),
      ) {
        let _ = result(code, false);
        continue;
      }
    }

    // 提取音檔
    if let Err(_) = handle(
      mp4_to_wav(code).await,
      &format!("Running mp4_to_wav for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // 生成字幕
    if subtitle {
      if let Err(_) = handle(
        gen_subtitle(code).await,
        &format!("Running gen_subtitle for code: {}", code),
      ) {
        let _ = result(code, false);
        continue;
      }
    }

    // 生成頭像模擬影片
    if let Err(_) = handle(
      run_gen_video_python(code, remove_bg).await,
      &format!("Running run_gen_video_python for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // 合成生成片段
    if let Err(_) = handle(
      merge_avatar_video_chunks(code).await,
      &format!("Running mp4_to_wav for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // 合成原影片與生成的頭像
    if let Err(_) = handle(
      merge_video_and_avatar_video(code, x, y, &shape).await,
      &format!("Running merge_video_and_avatar_video for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    }

    // 設定影片狀態為'Finish'
    if let Err(_) = handle(
      database::update_video_status(code, Finish),
      &format!("Updating video status to 'Finish' for code: {}", code),
    ) {
      let _ = result(code, false);
      continue;
    };

    if subtitle {
      // let subtitles = handle(
      //   database::get_subtitles(code),
      //   &format!("Getting subtitles for code: {}", code),
      // )?;

      let video_path: String;
      let output_path: String;
      let subtitles_path: String;

      if let Ok(p) = handle(get_file_path(code, SUBS_FILE), "Inserting subtitles_path") {
        subtitles_path = p;
      } else {
        let _ = result(code, false);
        continue;
      };

      if let Ok(p) = handle(get_file_path(code, RESULT_FILE), "Inserting video_path") {
        video_path = p;
      } else {
        let _ = result(code, false);
        continue;
      };

      if let Ok(p) = handle(
        create_file(code, RESULT_WITH_SUBS_FILE),
        "Inserting output_path",
      ) {
        output_path = p;
      } else {
        let _ = result(code, false);
        continue;
      };

      let mut data = HashMap::new();

      data.insert("subtitle_path", subtitles_path);
      data.insert("video_path", video_path);
      data.insert("output_path", output_path);

      let response = handle(
        make_request("http://localhost:5000/merge_video_and_subtitle", &data).await,
        "Making request",
      )
      .unwrap();

      if response.status().is_success() {
        log::info!("Python merge video and subtitle subtitle success");
        let _ = result(code, true);
      } else {
        let _ = result(code, false);
        continue;
      }
    } else {
      let _ = result(code, true);
    }

    log::info!("Video generation completed for code: {}", code);
  }
}

pub async fn start_merge_subs_worker(mut rx: Receiver<worker::MergeSubsRequest>) {
  log::info!("Starting merge subtitles worker!");

  while let Some(request) = rx.recv().await {
    log::info!(
      "Received a request to merge subtitles for code: {}",
      request.code
    );

    let code = &request.code;
    let task: Task;

    match handle(
      database::get_task_info(code),
      &format!("Getting task info for code: {}", code),
    ) {
      Ok(t) => task = t,
      Err(_) => {
        let _ = result(code, false);
        continue;
      }
    }

    match (task.subs_status, task.video_status) {
      (Finish, Finish) => {
        if let Err(_) = handle(
          merge_video_and_subtitle(code).await,
          &format!("Running merge_video_and_subtitle for code: {}", code),
        ) {
          let _ = result(code, false);
          continue;
        }
      }
      (_, _) => continue,
    }

    let _ = result(code, true);
    log::info!("Video merging completed for code: {}", code);
  }
}

fn result(code: &str, success: bool) -> Result<(), Error> {
  if success {
    // 設定任務狀態為'Finish'
    handle(
      database::update_task_status(code, Finish),
      &format!("Updating task status to 'Finish' for code: {}", code),
    )?;
  } else {
    // 設定任務狀態為'Fail'
    handle(
      database::update_task_status(code, Fail),
      &format!("Updating task status to 'Fail' for code: {}", code),
    )?;
  }

  // 如果有設定email就進行通知
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

  // 刪除不必要檔案
  handle(
    delete_file_in_dir(code),
    &format!("Deleting file in directoey '{}'", code),
  )?;

  if success {
    log::info!("Task of code: {} completed", code);
  } else {
    log::info!("Task of code: {} failed", code);
  }
  Ok(())
}
