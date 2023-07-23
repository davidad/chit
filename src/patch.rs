use crate::id::*;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use tinyvec::TinyVec;

#[derive(Default, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct UuidSetPatch {
  pub deletions: BTreeSet<Uuid>,
  pub merges: HashMap<Uuid, Uuid>,
  pub additions: BTreeSet<Uuid>,
}

impl UuidSetPatch {
  pub fn clear(&mut self) {
    self.deletions.clear();
    self.merges.clear();
    self.additions.clear();
  }
  pub fn is_empty(&self) -> bool {
    self.deletions.is_empty() && self.merges.is_empty() && self.additions.is_empty()
  }
}

pub type UniversePatch = UuidSetPatch;

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub enum AdditionKind {
  NewSort,
  NewEntity(Uuid),
}

pub type AdditionKinds = Vec<AdditionKind>;

#[derive(Default, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct ContextPatch {
  pub deletions: BTreeSet<Vec<String>>,
  pub additions: HashMap<Vec<String>, Uuid>,
}

#[derive(Default, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct Patch {
  pub target_commit: Uuid,
  pub source_commits: TinyVec<[Uuid; 2]>,
  pub universe_patch: UniversePatch,
  pub addition_kinds: AdditionKinds,
  pub context_patch: ContextPatch,
}

impl Patch {
  pub fn clear(&mut self) {
    self.target_commit = Uuid::nil();
    self.source_commits.clear();
    self.universe_patch.clear();
  }
  pub fn is_empty(&self) -> bool {
    self.universe_patch.is_empty()
  }
}
