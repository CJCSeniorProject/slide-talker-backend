mod logger;
mod utils;
mod video;
use video::api::{download, gen_video, get_video, set_email};

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

#[tokio::main]
async fn main() {
  logger::init_logger();

  let (tx, rx) = tokio::sync::mpsc::channel(100);
  tokio::spawn(video::start_worker(rx));

  let server = rocket::build()
    .register("/", catchers![handle_unprocessable_entity])
    .mount("/", routes![gen_video, set_email, get_video, download])
    .attach(CORS)
    .manage(tx.clone())
    .launch();

  tokio::select! {
      _ = server => {},
      _ = tokio::signal::ctrl_c() => {},
  }
}
