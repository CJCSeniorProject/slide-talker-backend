use crate::{model::task, utils::*};
use rusqlite::{params, Connection, Result, ToSql};

fn handle_error<T>(result: Result<T, rusqlite::Error>, title: &str) -> Result<T, String> {
  result.map_err(|e| {
    let err_msg = format!("{} failed with error: {:?}", title, e);
    log::error!("{}", err_msg);
    err_msg
  })
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

pub fn insert_task(code: &str) -> Result<(), String> {
  log::info!("Inserting task with code: {}", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  handle_error(
    conn.execute(
      "INSERT INTO task (code, status, date) VALUES (?1, ?2, ?3)",
      params![code, "Processing", get_date()],
    ),
    "Insert Operation",
  )?;

  log::info!("Insertion completed successfully");
  Ok(())
}

pub fn get_task_status(code: &str) -> Result<task::Status, String> {
  log::info!("Getting task status with code: {}", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  let mut stmt = handle_error(
    conn.prepare("SELECT status FROM task WHERE code = ?1"),
    "Select Operation",
  )?;
  let mut rows = handle_error(stmt.query(params![code]), "Query Operation")?;

  let row = handle_error(rows.next(), "Find Next Operation")?;

  if let Some(row) = row {
    let status = handle_error(row.get(0), "Get Row Data Operation")?;
    Ok(status)
  } else {
    Err("No task found".to_string())
  }
}

pub fn get_task_email(code: &str) -> Result<String, String> {
  log::info!("Getting task email with code: {}", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  let mut stmt = handle_error(
    conn.prepare("SELECT email FROM task WHERE code = ?1"),
    "Select Operation",
  )?;
  let mut rows = handle_error(stmt.query(params![code]), "Query Operation")?;

  let row = handle_error(rows.next(), "Find Next Operation")?;

  //[Error] Get Row Data Operation failed with error: InvalidColumnType(0, "email", Null)
  if let Some(row) = row {
    let email = handle_error(row.get(0), "Get Row Data Operation")?;
    Ok(email)
  } else {
    Err("None".to_string())
  }
}

pub fn update_task_status(code: &str, status: task::Status) -> Result<(), String> {
  log::info!("Updating task status with code: {}", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  handle_error(
    conn.execute(
      "UPDATE task SET status = ?1 WHERE code = ?2",
      params![status, code],
    ),
    "Update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn update_task_email(code: &str, email: &String) -> Result<(), String> {
  log::info!("Updating task email with code: {}", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  handle_error(
    conn.execute(
      "UPDATE task SET email = ?1 WHERE code = ?2",
      params![email, code],
    ),
    "Update Operation",
  )?;

  log::info!("Update completed successfully");
  Ok(())
}

pub fn delete_task_by_code(code: &str) -> Result<(), String> {
  log::info!("Deleting task in database by code");
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  handle_error(
    conn.execute("DELETE FROM task WHERE code = ?1", params![code]),
    "DELETE Operation",
  )?;

  log::info!("Deletion of task in database by code completed");
  Ok(())
}

pub fn search_task_by_date() -> Result<Vec<String>, String> {
  log::info!("Seaching task in database by date");
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  let mut stmt = handle_error(
    conn.prepare("SELECT code FROM task WHERE date <= ?1"),
    "Select Operation",
  )?;
  let mut rows = handle_error(stmt.query(params![get_last_week()]), "Query Operation")?;

  let mut codes = Vec::new();
  while let Some(row) = handle_error(rows.next(), "Find Next Operation")? {
    let code = handle_error(row.get(0), "Get Row Data Operation")?;
    codes.push(code);
  }

  log::info!("Seaching of task in database by date completed");
  Ok(codes)
}

pub fn check_code_exists(code: &str) -> Result<bool, String> {
  log::info!("Checking if code: {} exists in the database.", code);
  let conn = handle_error(Connection::open("./slidetalker.db3"), "DB Connect")?;

  let count: i64 = handle_error(
    conn.query_row(
      "SELECT COUNT(*) FROM task WHERE code = ?1",
      params![code],
      |row| row.get(0),
    ),
    "Select Operation",
  )?;

  if count > 0 {
    log::debug!("code:{} exists", code);
    Ok(true)
  } else {
    log::debug!("code:{} does not exist", code);
    Ok(false)
  }
}
