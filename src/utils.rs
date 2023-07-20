use rand::Rng;
use reqwest;
use std::{
  collections::HashMap,
  fs::{self, File},
  path::Path,
  time::{SystemTime, UNIX_EPOCH},
};

pub fn get_file_path(code: &str, filename: &str) -> Result<String, String> {
  log::info!("Getting path of file '{}' for code: {}", filename, code);
  let path = format!(
    "/home/lab603/Documents/slide_talker_backend/tmp/{}/{}",
    code, filename
  );
  log::debug!("path={}", path);

  let path_str = Path::new(path.as_str());
  if path_str.exists() {
    log::info!("File '{}' found for code: {}", filename, code);
    return Ok(path);
  }
  let err_msg = format!("File '{}' not found for code: {}", filename, code);
  log::error!("{}", err_msg);
  Err(err_msg)
}

pub fn create_file(code: &str, filename: &str) -> Result<String, String> {
  log::info!("Creating file '{}' for code: {}", filename, code);

  let path = format!(
    "/home/lab603/Documents/slide_talker_backend/tmp/{}/{}",
    code, filename
  );
  log::debug!("path={}", path);

  File::create(&path).map_err(|e| {
    let err_msg = format!("Failed to create file '{}': {}", path, e);
    log::error!("{}", err_msg);
    err_msg
  })?;

  log::info!("File created");
  Ok(path)
}

pub fn create_dir(code: &str, dirname: &str) -> Result<String, String> {
  log::info!("Creating directory '{}' for code: {}", dirname, code);

  let mut path = format!("/home/lab603/Documents/slide_talker_backend/tmp/{}", code);

  if !dirname.is_empty() {
    path = format!("{}/{}", path, dirname);
  }
  log::debug!("path={}", path);
  fs::create_dir_all(&path).map_err(|e| {
    let err_msg = format!("Failed to create directory '{}': {}", path, e);
    log::error!("{}", err_msg);
    err_msg
  })?;
  log::info!("Directory created");
  Ok(path)
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
) -> Result<reqwest::Response, String> {
  let client = reqwest::Client::new();
  client.post(url).json(map).send().await.map_err(|e| {
    let err_msg = format!("Request failed with error: {:?}", e);
    log::error!("{}", err_msg);
    err_msg
  })
}
