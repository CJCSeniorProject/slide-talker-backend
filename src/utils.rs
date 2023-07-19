use rand::Rng;
use reqwest;
use std::{
  collections::HashMap,
  time::{SystemTime, UNIX_EPOCH},
};

pub fn get_file_path(code: &str, filename: &str) -> String {
  return format!(
    "/home/lab603/Documents/slide_talker_backend/tmp/{}/{}",
    code, filename
  );
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
