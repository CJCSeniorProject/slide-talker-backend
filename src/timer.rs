use crate::{database, utils::*};
use std::{env, fs, time};
use tokio::time::sleep;

pub async fn start() {
  loop {
    let tomorrow_midnight = get_tomorrow_midnight();
    let duration = tomorrow_midnight - get_datetime();
    let std_duration = duration.to_std().unwrap();
    // let std_duration = time::Duration::from_secs(3);

    sleep(std_duration).await;
    let _ = delete_logfile();
    let _ = delete_data();
  }
}

pub fn delete_logfile() -> Result<(), String> {
  log::info!("Deleting logfiles");

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = root + "/logfiles";
  log::debug!("path={}", path);

  let dir = fs::read_dir(path);

  match dir {
    Ok(entries) => {
      for entry in entries {
        log::debug!("entry={:?}", entry);

        match entry {
          Ok(entry) => {
            let file_name = entry.file_name().to_string_lossy().to_string();

            if let Some(date_str) = file_name.split('.').next() {
              if let Ok(date) = date_from_string(date_str) {
                // 刪除7天前的logfile
                if date <= get_last_week() {
                  if let Err(e) = fs::remove_file(entry.path()) {
                    log::error!("Failed to remove file '{}': {}", file_name, e);
                  }
                }
              }
            }
          }
          Err(e) => {
            log::error!("Failed to read directory entry: {:?}", e);
            return Err("err".to_string());
          }
        }
      }
    }
    Err(e) => {
      log::error!("Failed to read directory: {:?}", e);
      return Err("err".to_string());
    }
  }
  Ok(())
}

pub fn delete_data() -> Result<(), String> {
  if let Ok(codes) = database::search_task_by_date() {
    for code in codes.clone() {
      let _ = delete_code_dir(&code);
      database::delete_task_by_code(&code)?;
    }
  } else {
    log::error!("");
    return Err("Failed at database::search_task_by_date".to_string());
  }
  Ok(())
}
