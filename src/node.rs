use crate::{prelude::*};
pub enum NodeType {
  Follower,
  Leader,
  Candidate
}

pub struct NodeInfo {
  term: i32,
  leader: Option<String>,
  node_type: NodeType,
  peers: Vec<String>,
  log: Vec<(i32, Operation)>,
  value: String
}