use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

pub enum Status {
  Fail,
  Processing,
  Finish,
}

impl ToString for Status {
  fn to_string(&self) -> String {
    match self {
      Status::Fail => "Fail".to_string(),
      Status::Processing => "Processing".to_string(),
      Status::Finish => "Finish".to_string(),
    }
  }
}

impl ToSql for Status {
  fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
    let value = match self {
      Status::Fail => Value::Text("fail".to_string()),
      Status::Processing => Value::Text("Processing".to_string()),
      Status::Finish => Value::Text("finish".to_string()),
    };
    Ok(ToSqlOutput::Owned(value))
  }
}

impl FromSql for Status {
  fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
    match value.as_str() {
      Ok("fail") => Ok(Status::Fail),
      Ok("Processing") => Ok(Status::Processing),
      Ok("finish") => Ok(Status::Finish),
      _ => Err(FromSqlError::InvalidType),
    }
  }
}
