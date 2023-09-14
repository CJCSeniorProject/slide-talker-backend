use chrono::NaiveTime;
use rocket::{
  form::{self, Error},
  FromForm,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, FromForm, Serialize, Deserialize)]
pub struct Subtitle {
  #[field(validate = len(1..))]
  pub text: String,
  #[field(default = 32)]
  pub fontsize: u32,
  #[field(default = "white")]
  pub color: String,
  #[field(default = "./NotoSansCJK-Regular.ttc")]
  pub font: String,
  #[field(validate = validate_time())]
  pub start_time: String,
  #[field(validate = validate_time())]
  pub end_time: String,
}

#[derive(Debug, FromForm, Serialize, Deserialize)]
pub struct Request {
  pub subtitles: Vec<Subtitle>,
}

fn validate_time<'v>(time: &str) -> form::Result<'v, ()> {
  match NaiveTime::parse_from_str(&time.replace(",", "."), &"%H:%M:%S%.f") {
    Ok(_) => Ok(()),
    Err(_) => Err(Error::validation("Incorrect Time Format").into()),
  }
}

#[test]
fn test_time_validate() {
  let test1 = "00:00:10,500";
  let test2 = "00:00:10";
  let test3 = "";
  let test4 = "00-00-10-500";

  assert_eq!(
    NaiveTime::parse_from_str(&test1.replace(",", "."), "%H:%M:%S%.f"),
    Ok(NaiveTime::from_hms_milli_opt(0, 0, 10, 500).unwrap())
  );
  assert_eq!(
    NaiveTime::parse_from_str(&test2.replace(",", "."), "%H:%M:%S%.f"),
    Ok(NaiveTime::from_hms_milli_opt(0, 0, 10, 0).unwrap())
  );
  assert!(NaiveTime::parse_from_str(&test3.replace(",", "."), "%H:%M:%S%.f").is_err());
  assert!(NaiveTime::parse_from_str(&test4.replace(",", "."), "%H:%M:%S%.f").is_err());
}
