use super::common::*;
use crate::timer::*;

#[test]
fn test_delete_data() {
  let tasks_to_delete = vec![
    ("task1", get_last_week()),
    ("task2", get_last_week()),
    ("task3", get_date()),
  ];
  for (code, date) in tasks_to_delete {
    create_code_dir(code);
    insert_task_with_date(code, date);
  }
  delete_data().expect("Failed to delete data");
  assert_eq!(check_task_exists_in_database("task1"), false);
  assert_eq!(check_task_exists_in_database("task3"), true);
  assert_eq!(check_codefile_exists_in_tmp("task1"), false);
  assert_eq!(check_codefile_exists_in_tmp("task3"), true);

  delete_task_by_code("task3");
  delete_code_dir("task3");
}

#[test]
fn test_delete_logfile() {
  create_logfile(get_last_week());
  delete_logfile().expect("Failed to delete data");
  assert_eq!(check_logfile_exists(get_last_week()), false);
}
