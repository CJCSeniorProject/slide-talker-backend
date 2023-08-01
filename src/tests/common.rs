pub use crate::model::*;
use crate::{database, utils};
use chrono::NaiveDate;
use dotenv::dotenv;
use rusqlite::{params, Connection};
use std::{
  env,
  fs::{self, File},
  io::{self, Read, Write},
  path::{Path, PathBuf},
};

pub static BOUNDARY: &'static str = "--------------------------------XYZ";

fn copy_image_to_temp(path: &str) -> PathBuf {
  let root = env::var("ROOT").expect("Failed to get root path");
  let image_path = format!("{}/testdata/{}", root, path);
  let temp_dir = env::temp_dir();
  let image_file_name = Path::new(&image_path).file_name().unwrap();
  let temp_image_path = temp_dir.join(image_file_name);

  fs::copy(image_path, &temp_image_path).expect("Failed to copy image to temp directory.");

  temp_image_path
}

fn copy_video_to_temp(path: &str) -> PathBuf {
  let root = env::var("ROOT").expect("Failed to get root path");
  let video_path = format!("{}/testdata/{}", root, path);
  let temp_dir = env::temp_dir();
  let video_file_name = Path::new(&video_path).file_name().unwrap();
  let temp_video_path = temp_dir.join(video_file_name);

  fs::copy(video_path, &temp_video_path).expect("Failed to copy video to temp directory.");

  temp_video_path
}

pub fn create_form_data(
  boundary: &str,
  video_path: &str,
  avatar_path: &str,
  x: &str,
  y: &str,
  shape: &str,
  subtitle: &str,
) -> io::Result<Vec<u8>> {
  let mut data = Vec::new();
  // start
  write!(data, "--{}\r\n", boundary)?;

  // video data
  if video_path != "" {
    write!(
      data,
      "Content-Disposition: form-data; name=\"video\"; filename=\"video.mp4\"\r\n"
    )?;

    let content_type = match Path::new(video_path)
      .extension()
      .and_then(|ext| ext.to_str())
    {
      Some("mp4") => "video/mp4",
      Some("mov") => "video/quicktime",
      Some("jpg") => "image/jpeg",
      Some("jpeg") => "image/jpeg",
      _ => "application/octet-stream",
    };

    write!(data, "Content-Type: {}\r\n", content_type)?;
    write!(data, "Content-Type: video/mp4\r\n")?;
    write!(data, "\r\n")?;

    let video_path = copy_video_to_temp(video_path);
    let mut video_file = File::open(video_path.as_path())?;
    video_file.read_to_end(&mut data)?;
    write!(data, "\r\n")?;

    write!(data, "--{}\r\n", boundary)?;
  }

  // avatar data
  if avatar_path != "" {
    write!(
      data,
      "Content-Disposition: form-data; name=\"avatar\"; filename=\"avatar.jpg\"\r\n"
    )?;
    let content_type = match Path::new(avatar_path)
      .extension()
      .and_then(|ext| ext.to_str())
    {
      Some("mp4") => "video/mp4",
      Some("mov") => "video/quicktime",
      Some("jpg") => "image/jpeg",
      Some("jpeg") => "image/jpeg",
      _ => "application/octet-stream",
    };

    write!(data, "Content-Type: {}\r\n", content_type)?;
    write!(data, "\r\n")?;

    let avatar_path = copy_image_to_temp(avatar_path);
    let mut avatar_file = File::open(avatar_path.as_path())?;
    avatar_file.read_to_end(&mut data)?;
    write!(data, "\r\n")?;

    write!(data, "--{}\r\n", boundary)?;
  }

  // x and y fields
  write!(data, "Content-Disposition: form-data; name=\"x\"\r\n")?;
  write!(data, "\r\n")?;
  write!(data, "{}", x)?;
  write!(data, "\r\n")?;

  write!(data, "--{}\r\n", boundary)?;
  write!(data, "Content-Disposition: form-data; name=\"y\"\r\n")?;
  write!(data, "\r\n")?;
  write!(data, "{}", y)?;
  write!(data, "\r\n")?;

  // shape field
  write!(data, "--{}\r\n", boundary)?;
  write!(data, "Content-Disposition: form-data; name=\"shape\"\r\n")?;
  write!(data, "\r\n")?;
  write!(data, "{}", shape)?;
  write!(data, "\r\n")?;

  // subtitle field
  write!(data, "--{}\r\n", boundary)?;
  write!(
    data,
    "Content-Disposition: form-data; name=\"subtitle\"\r\n"
  )?;
  write!(data, "\r\n")?;
  write!(data, "{}", subtitle)?;
  write!(data, "\r\n")?;

  // end
  write!(data, "--{}--\r\n", boundary)?;

  Ok(data)
}

pub fn insert_task_with_status(code: &str, status: task::Status) {
  database::init_db();
  let conn = Connection::open("./slidetalker.db3").expect("Failed to open ./slidetalker.db3");
  delete_task_by_code(code);

  conn
    .execute(
      "INSERT INTO task (code, status, date) VALUES (?1, ?2, ?3)",
      params![code, status, utils::get_date()],
    )
    .expect("Failed to insert task");
}

pub fn insert_task_with_date(code: &str, date: NaiveDate) {
  database::init_db();
  let conn = Connection::open("./slidetalker.db3").expect("Failed to open ./slidetalker.db3");
  delete_task_by_code(code);

  conn
    .execute(
      "INSERT INTO task (code, status, date) VALUES (?1, ?2, ?3)",
      params![code, task::Status::Finish, date],
    )
    .expect("Failed to insert task");
}

pub fn delete_task_by_code(code: &str) {
  database::delete_task_by_code(code).expect("Failed to delete task by code");
}

pub fn delete_code_dir(code: &str) {
  utils::delete_code_dir(code).expect("Failed to delete code directory");
}

pub fn create_logfile(date: NaiveDate) {
  dotenv().ok();
  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!(
    "{}/logfiles/{}.txt",
    root,
    date.format("%Y-%m-%d").to_string()
  );
  File::create(&path).expect("Failed to create file");
}

pub fn create_code_dir(code: &str) {
  dotenv().ok();
  utils::create_code_dir(code).expect("Failed to create code directory");
}

pub fn get_date() -> NaiveDate {
  utils::get_date()
}

pub fn get_last_week() -> NaiveDate {
  utils::get_last_week()
}

pub fn check_task_exists_in_database(code: &str) -> bool {
  database::check_code_exists(code).expect("Failed to check code exists")
}

pub fn check_codefile_exists_in_tmp(code: &str) -> bool {
  dotenv().ok();
  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/tmp/{}", root, code);
  let file_path = Path::new(path.as_str());
  file_path.exists()
}

pub fn check_logfile_exists(date: NaiveDate) -> bool {
  dotenv().ok();
  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!(
    "{}/logfiles/{}.txt",
    root,
    date.format("%Y-%m-%d").to_string()
  );
  let file_path = Path::new(path.as_str());
  file_path.exists()
}
