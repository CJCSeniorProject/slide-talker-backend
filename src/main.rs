mod api;
mod controller;
mod database;
mod logger;
mod model;
mod timer;
mod utils;
mod worker;

#[cfg(test)]
mod tests;

use api::*;
use dotenv::dotenv;
use rocket::{
  self, catch, catchers,
  fairing::{Fairing, Info, Kind},
  http::Header,
  routes, {Request, Response},
};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to responses",
      kind: Kind::Response,
    }
  }

  async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new(
      "Access-Control-Allow-Methods",
      "POST, GET, PATCH, OPTIONS",
    ));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

#[catch(422)]
fn handle_unprocessable_entity(_: &Request) -> &'static str {
  "Unprocessable Entity"
}

#[catch(499)]
fn handle_processing(_: &Request) -> &'static str {
  "Processing"
}

#[catch(498)]
fn handle_code_undefind(_: &Request) -> &'static str {
  "Code undefind"
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  logger::init_logger(log::LevelFilter::Info);
  database::init_db();

  tokio::spawn(timer::start());
  let (mtx, mrx) = tokio::sync::mpsc::channel::<model::worker::MergeSubsRequest>(100);
  let (tx, rx) = tokio::sync::mpsc::channel::<model::worker::GenVideoRequest>(100);
  tokio::spawn(worker::start_gen_video_worker(rx, mtx.clone()));
  tokio::spawn(worker::start_merge_subs_worker(mrx));

  let server = rocket::build()
    .register(
      "/",
      catchers![
        handle_unprocessable_entity,
        handle_processing,
        handle_code_undefind
      ],
    )
    .mount(
      "/",
      routes![
        gen_video,
        set_email,
        check_task_status,
        download,
        get_file_path_for_code,
        set_subtitle
      ],
    )
    .attach(CORS)
    .manage(model::worker::Sender {
      gen_sender: tx,
      merge_sender: mtx,
    })
    .launch();

  tokio::select! {
      _ = server => {},
      _ = tokio::signal::ctrl_c() => {},
  }
}
