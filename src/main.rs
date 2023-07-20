mod api;
mod controller;
mod database;
mod logger;
mod model;
mod utils;
mod worker;

use api::{download, gen_video, get_file_path, get_video, set_email, test_fn};

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

#[tokio::main]
async fn main() {
  logger::init_logger(log::LevelFilter::Info);
  database::init_db();

  let (tx, rx) = tokio::sync::mpsc::channel(100);
  tokio::spawn(worker::start_worker(rx));

  let server = rocket::build()
    .register(
      "/",
      catchers![handle_unprocessable_entity, handle_processing],
    )
    .mount(
      "/",
      routes![
        gen_video,
        set_email,
        get_video,
        download,
        get_file_path,
        test_fn
      ],
    )
    .attach(CORS)
    .manage(tx.clone())
    .launch();

  tokio::select! {
      _ = server => {},
      _ = tokio::signal::ctrl_c() => {},
  }
}
