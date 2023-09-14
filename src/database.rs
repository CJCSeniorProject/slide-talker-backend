use crate::{
  model::{
    subtitle::{self, Subtitle},
    task::{
      Status::{self, Finish, Processing},
      Task,
    },
  },
  utils::*,
};
use rusqlite::{params, Connection, Result};
use std::env;

fn connect_to_db() -> Result<Connection, Error> {
  let root = env::var("ROOT").expect("Failed to get root path");
  let path = format!("{}/slidetalker.db3", root);
  log::debug!("path={}", path);

  handle(Connection::open(path), "DB Connect")
}

pub fn init_db() {
  log::info!("Initializing db");
  let conn = Connection::open("./slidetalker.db3").unwrap_or_else(|e| {
    log::error!("Failed to open SQLite connection: {}", e);
    panic!("Failed to open SQLite connection");
  });

  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS task (
      code VARCHAR(10) NOT NULL,
      email VARCHAR(64),
      status VARCHAR(16) NOT NULL,
      date DATE NOT NULL,
      subs_status VARCHAR(16) NOT NULL,
      subtitles TEXT,
      video_status VARCHAR(16) NOT NULL,
      PRIMARY KEY (code),
      UNIQUE (code)
    );",
      (), // empty list of parameters.
    )
    .unwrap_or_else(|e| {
      log::error!("Failed to create table: {}", e);
      panic!("Failed to create table");
    });

  log::info!("Initialization completed successfully");
}

pub fn insert_task(code: &str, subs: bool) -> Result<(), Error> {
  log::info!("Inserting task with code: {}", code);
  let conn = connect_to_db()?;

  let subs_status = match subs {
    true => Processing,
    false => Finish,
  };

  handle(
    conn.execute(
      "INSERT INTO task (code, status, date, subs_status, video_status) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![code, Processing.to_string(), get_date(), subs_status, Processing.to_string()],
    ),
    "Executeing insert operation",
  )?;

  log::info!("Insertion completed successfully");
  Ok(())
}

pub fn get_task_info(code: &str) -> Result<Task, Error> {
  log::info!("Getting task info for code: {}", code);
  let conn = connect_to_db()?;

  let mut stmt = handle(
    conn.prepare("SELECT * FROM task WHERE code = ?1"),
    "Preparing select operation",
  )?;
  let mut rows = handle(stmt.query(params![code]), "Querying operation")?;
  let row = handle(rows.next(), "Finding next row")?;

  if let Some(row) = row {
    let code: String = handle(row.get(0), "Getting row data operation")?;
    let status: Status = handle(row.get(2), "Getting row data operation")?;
    let subs_status: Status = handle(row.get(4), "Getting row data operation")?;
    let video_status: Status = handle(row.get(6), "Getting row data operation")?;
    Ok(Task {
      code: code,
      status: status,
      subs_status: subs_status,
      video_status: video_status,
    })
  } else {
    Err(Error::new(ErrorKind::Other, "No task found"))
  }
}

pub fn get_subtitles(code: &str) -> Result<Vec<Subtitle>, Error> {
  log::info!("Getting task status with code: {}", code);
  let conn = connect_to_db()?;

  let mut stmt = handle(
    conn.prepare("SELECT subtitles FROM task WHERE code = ?1"),
    "Preparing select operation",
  )?;
  let mut rows = handle(stmt.query(params![code]), "Querying operation")?;
  let row = handle(rows.next(), "Finding next row")?;

  if let Some(row) = row {
    let json_str: String = handle(row.get(0), "Getting row data operation")?;

    // 将 JSON 数据解析为 MyData 结构体
    let data: subtitle::Request =
      serde_json::from_str(&json_str).expect("JSON deserialization failed");
    Ok(data.subtitles)
  } else {
    Err(Error::new(ErrorKind::Other, "No task found"))
  }
}

pub fn get_task_email(code: &str) -> Result<Option<String>, Error> {
  log::info!("Getting task email with code: {}", code);
  let conn = connect_to_db()?;

  let mut stmt = handle(
    conn.prepare("SELECT email FROM task WHERE code = ?1"),
    "Preparing select operation",
  )?;
  let mut rows = handle(stmt.query(params![code]), "Querying operation")?;

  let row = handle(rows.next(), "Finding next row")?;

  if let Some(row) = row {
    match row.get(0) {
      Ok(email) => Ok(Some(email)),
      Err(_) => Ok(None), // email = NULL
    }
  } else {
    Err(Error::new(ErrorKind::Other, "No task found"))
  }
}

pub fn update_task_status(code: &str, status: Status) -> Result<(), Error> {
  log::info!("Updating task status with code: {}", code);
  let conn = connect_to_db()?;

  handle(
    conn.execute(
      "UPDATE task SET status = ?1 WHERE code = ?2",
      params![status, code],
    ),
    "Executing update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn update_subtitles_status(code: &str, status: Status) -> Result<(), Error> {
  log::info!("Updating task status with code: {}", code);
  let conn = connect_to_db()?;

  handle(
    conn.execute(
      "UPDATE task SET subs_status = ?1 WHERE code = ?2",
      params![status, code],
    ),
    "Executing update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn update_video_status(code: &str, status: Status) -> Result<(), Error> {
  log::info!("Updating task status with code: {}", code);
  let conn = connect_to_db()?;

  handle(
    conn.execute(
      "UPDATE task SET video_status = ?1 WHERE code = ?2",
      params![status, code],
    ),
    "Executing update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn update_task_email(code: &str, email: &str) -> Result<(), Error> {
  log::info!("Updating task email with code: {}", code);
  let conn = connect_to_db()?;

  handle(
    conn.execute(
      "UPDATE task SET email = ?1 WHERE code = ?2",
      params![email, code],
    ),
    "Executing update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn update_task_subtitles(code: &str, subs: &Vec<Subtitle>) -> Result<(), Error> {
  log::info!("Updating task email with code: {}", code);
  let conn = connect_to_db()?;

  let json_str = serde_json::to_string(&subs).expect("JSON serialization failed");
  handle(
    conn.execute(
      "UPDATE task SET subtitles = ?1 WHERE code = ?2",
      params![json_str, code],
    ),
    "Executing update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn delete_task_by_code(code: &str) -> Result<(), Error> {
  log::info!("Deleting task in database by code");
  let conn = connect_to_db()?;

  handle(
    conn.execute("DELETE FROM task WHERE code = ?1", params![code]),
    "Executing delete operation",
  )?;

  log::info!("Deletion of task in database by code completed");
  Ok(())
}

pub fn search_task_by_date() -> Result<Vec<String>, Error> {
  // 搜尋存放超過一周的資料
  log::info!("Seaching task in database by date");
  let conn = connect_to_db()?;

  let mut stmt = handle(
    conn.prepare("SELECT code FROM task WHERE date <= ?1"),
    "Preparing select operation",
  )?;
  let mut rows = handle(stmt.query(params![get_last_week()]), "Querying operation")?;

  let mut codes = Vec::new();
  while let Some(row) = handle(rows.next(), "Finding next row")? {
    let code = handle(row.get(0), "Getting row data operation")?;
    codes.push(code);
  }

  log::info!("Seaching of task in database by date completed");
  Ok(codes)
}

pub fn check_code_exists(code: &str) -> Result<bool, Error> {
  log::info!("Checking if code: {} exists in the database.", code);
  let conn = connect_to_db()?;

  let count: i64 = handle(
    conn.query_row(
      "SELECT COUNT(*) FROM task WHERE code = ?1",
      params![code],
      |row| row.get(0),
    ),
    "Querying select operation",
  )?;

  if count > 0 {
    log::debug!("code: {} exists", code);
    Ok(true)
  } else {
    log::debug!("code: {} does not exist", code);
    Ok(false)
  }
}
