use crate::{database, utils::*};
use std::{env, fs};
use tokio::time::sleep;

pub async fn start() {
  loop {
    let tomorrow_midnight = get_tomorrow_midnight();
    let duration = tomorrow_midnight - get_datetime();
    let std_duration = duration.to_std().unwrap();
    // let std_duration = std::time::Duration::from_secs(3);

    sleep(std_duration).await;
    let _ = handle(delete_logfile(), "Deleting logfile");
    let _ = handle(delete_data(), "Deleting data");
  }
}

pub fn delete_logfile() -> Result<(), Error> {
  log::info!("Deleting logfiles");

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = root + "/logfiles";
  log::debug!("path={}", path);

  let entries = handle(
    fs::read_dir(&path),
    &format!("Reading directory '{}'", path),
  )?;

  for entry in entries {
    match entry {
      Ok(entry) => {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(date_str) = file_name.split('.').next() {
          if let Ok(date) = date_from_string(date_str) {
            // 刪除7天前的logfile
            if date <= get_last_week() {
              let _ = handle(
                fs::remove_file(entry.path()),
                &format!("Removing file '{}'", entry.path().display()),
              );
            }
          }
        }
      }
      Err(e) => {
        log::warn!("Failed to read directory entry: {:?}", e);
      }
    }
  }

  Ok(())
}

pub fn delete_data() -> Result<(), Error> {
  // 刪除存放超過一周的資料
  let codes = handle(database::search_task_by_date(), "Searching task by date")?;

  for code in codes.clone() {
    let _ = delete_code_dir(&code);
    let _ = database::delete_task_by_code(&code);
  }

  Ok(())
}
