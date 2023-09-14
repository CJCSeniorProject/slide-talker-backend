// use super::common::*;
// use crate::database::*;
// use rusqlite::{params, Connection};

// #[test]
// fn test_delete_task_by_code() {
//   let conn = Connection::open("./slidetalker.db3").expect("Failed to open ./slidetalker.db3");
//   let code = "test";
//   let result = conn.execute("DELETE FROM task WHERE code = ?1", params![code]);
//   assert!(result.is_ok());
// }
