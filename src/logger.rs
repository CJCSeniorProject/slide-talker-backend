use chrono::Local;
use env_logger::{fmt::Color, Target};
use log::{Level, LevelFilter};
use std::{fs::File, io::Write};

pub fn init_logger() {
  // let file = Box::new(File::create("log.txt").expect("Can't create file"));
  // let target = Target::Pipe(file);

  env_logger::Builder::new()
    .target(Target::Stdout)
    .filter(None, LevelFilter::Debug)
    .format(|buf, record| {
      let mut bold = buf.style();
      bold.set_bold(true);

      let level_style = match record.level() {
        Level::Error => bold.set_color(Color::Red),
        Level::Warn => bold.set_color(Color::Yellow),
        Level::Info => bold.set_color(Color::Green),
        Level::Debug => bold.set_color(Color::Blue),
        Level::Trace => bold.set_color(Color::White),
      };

      writeln!(
        buf,
        "[{} {}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        level_style.value(record.level()),
        record.args()
      )
    })
    .format_level(true)
    .init();
}
