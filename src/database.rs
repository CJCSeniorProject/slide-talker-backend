use crate::{model::task, utils::*};
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

pub fn insert_task(code: &str) -> Result<(), Error> {
  log::info!("Inserting task with code: {}", code);
  let conn = connect_to_db()?;

  handle(
    conn.execute(
      "INSERT INTO task (code, status, date) VALUES (?1, ?2, ?3)",
      params![code, "Processing", get_date()],
    ),
    "Executeing insert operation",
  )?;

  log::info!("Insertion completed successfully");
  Ok(())
}

pub fn get_task_status(code: &str) -> Result<task::Status, Error> {
  log::info!("Getting task status with code: {}", code);
  let conn = connect_to_db()?;

  let mut stmt = handle(
    conn.prepare("SELECT status FROM task WHERE code = ?1"),
    "Preparing select operation",
  )?;
  let mut rows = handle(stmt.query(params![code]), "Querying operation")?;

  let row = handle(rows.next(), "Finding next row")?;

  if let Some(row) = row {
    let status = handle(row.get(0), "Getting row data operation")?;
    Ok(status)
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

pub fn update_task_status(code: &str, status: task::Status) -> Result<(), Error> {
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
