use crate::id::*;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tinyvec::TinyVec;

#[derive(Default, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct UuidSetPatch {
  pub target_commit: Uuid,
  pub source_commits: TinyVec<[Uuid; 2]>,
  pub deletions: HashSet<Uuid>,
  pub additions: HashSet<Uuid>,
  pub merges: HashMap<Uuid, Uuid>,
  pub splits: HashMap<Uuid, Uuid>,
}

impl UuidSetPatch {
  pub fn clear(&mut self) {
    self.target_commit = Uuid::nil();
    self.source_commits.clear();
    self.deletions.clear();
    self.additions.clear();
    self.merges.clear();
  }
  pub fn is_empty(&self) -> bool {
    self.deletions.is_empty() && self.additions.is_empty() && self.merges.is_empty()
  }
}
