#[derive(Debug)]
pub struct GenVideoRequest {
  pub code: String,
  pub x: f32,
  pub y: f32,
  pub shape: String,
  pub remove_bg: bool,
  pub subtitle: bool,
}

#[derive(Debug)]
pub struct MergeSubsRequest {
  pub code: String,
}

pub struct Sender {
  pub gen_sender: tokio::sync::mpsc::Sender<GenVideoRequest>,
  pub merge_sender: tokio::sync::mpsc::Sender<MergeSubsRequest>,
}
