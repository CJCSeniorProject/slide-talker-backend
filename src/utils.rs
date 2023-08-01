use crate::model::constant::*;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use rand::Rng;
use reqwest;
pub use std::io::{Error, ErrorKind};
use std::{
  collections::HashMap,
  env,
  fs::{self, File},
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

pub fn handle<T, E>(result: Result<T, E>, msg: &str) -> Result<T, Error>
where
  E: std::error::Error,
{
  result.map_err(|e| {
    log::error!("{} failed with error: {:?}", msg, e);

    let err_msg = format!("{} failed", msg);
    Error::new(ErrorKind::Other, err_msg)
  })
}

pub fn get_file_path(code: &str, filename: &str) -> Result<String, Error> {
  log::debug!("Getting path of file '{}' for code: {}", filename, code);

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}/{}", root, code, filename);
  log::debug!("path={}", path);

  let file_path = Path::new(path.as_str());
  if file_path.exists() {
    log::debug!("File '{}' found for code: {}", filename, code);
    return Ok(path);
  }
  log::warn!("File '{}' not found for code: {}", filename, code);
  Err(Error::new(ErrorKind::Other, ""))
}

pub fn create_file(code: &str, filename: &str) -> Result<String, Error> {
  log::debug!("Creating file '{}' for code: {}", filename, code);

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}/{}", root, code, filename);
  log::debug!("path={}", path);

  handle(File::create(&path), &format!("Creating file '{}'", path))?;
  log::debug!("File created");
  Ok(path)
}

pub fn create_dir(code: &str, dirname: &str) -> Result<String, Error> {
  log::debug!("Creating directory '{}' for code: {}", dirname, code);

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}/{}", root, code, dirname);
  log::debug!("path={}", path);

  handle(
    fs::create_dir_all(&path),
    &format!("Creating directoey '{}'", path),
  )?;
  log::debug!("Directory created");

  Ok(path)
}

pub fn create_code_dir(code: &str) -> Result<String, Error> {
  log::debug!("Creating code directory for code: {}", code);

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}", root, code);
  log::debug!("path={}", path);

  handle(
    fs::create_dir_all(&path),
    &format!("Creating directoey '{}'", path),
  )?;
  log::debug!("Directory created");

  Ok(path)
}

pub fn delete_file_in_dir(code: &str) -> Result<(), Error> {
  log::info!("Deleting files in directory by code: {}", code);

  let gen = handle(get_file_path(code, "gen"), "Getting file path")?;
  handle(
    fs::remove_dir_all(&gen),
    &format!("Removing directory '{}'", gen),
  )?;

  let files_to_keep = [RESULT_FILE, RESULT_WITH_SUBS_FILE];
  let folder_path = get_file_path(code, "")?;

  let dir = handle(
    fs::read_dir(&folder_path),
    &format!("Reading file '{}'", folder_path),
  )?;

  for entry in dir {
    let entry = handle(
      entry,
      &format!("Reading entry in directory '{}'", folder_path),
    )?;

    let path = entry.path();
    let file_name = match path.file_name() {
      Some(name) => name.to_string_lossy().to_string(),
      None => continue,
    };
    log::debug!("file={}", file_name);

    if !files_to_keep.contains(&file_name.as_str()) {
      log::debug!("remove file={}", file_name);

      if path.is_dir() {
        handle(
          fs::remove_dir_all(&path),
          &format!("Removing directory '{}'", path.display()),
        )?;
      } else {
        handle(
          fs::remove_file(&path),
          &format!("Removing file '{}'", path.display()),
        )?;
      }
    }
  }
  log::info!("Deletion of files in directory by code: {} completed", code);
  Ok(())
}

pub fn delete_code_dir(code: &str) -> Result<(), Error> {
  log::info!("Deleting directory for code: {}", code);

  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}", root, code);
  log::debug!("path={}", path);

  let path_buf = PathBuf::from(&path);
  if path_buf.exists() {
    handle(
      fs::remove_dir_all(path),
      &format!("Removing directory '{}'", code),
    )?;
  } else {
    log::warn!("Directory for code {} not found, skipping deletion.", code);
  }

  log::info!("Directory for code {} deleted successfully", code);
  Ok(())
}

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

pub async fn make_request(
  url: &str,
  map: &HashMap<&str, String>,
) -> Result<reqwest::Response, Error> {
  let client = reqwest::Client::new();
  handle(client.post(url).json(map).send().await, "Requesting")
}

pub fn get_date() -> NaiveDate {
  Local::now().date_naive()
}

pub fn get_datetime() -> NaiveDateTime {
  Local::now().naive_local()
}

pub fn get_tomorrow_midnight() -> NaiveDateTime {
  let now = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
  let get_tomorrow_midnight = now + Duration::days(1);
  get_tomorrow_midnight
}

pub fn get_last_week() -> NaiveDate {
  let now = Local::now().date_naive();
  let seven_days_ago = now - Duration::days(7);
  seven_days_ago
}

pub fn date_from_string(date: &str) -> Result<NaiveDate, Error> {
  handle(
    NaiveDate::parse_from_str(date, "%Y-%m-%d"),
    &format!("Parsing date from str '{}'", date),
  )
}
