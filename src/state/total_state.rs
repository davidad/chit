use crate::id::*;
use crate::patch::*;
use indexmap::IndexSet;
use range_set_blaze::RangeSetBlaze;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tinyvec::TinyVec;

pub type Universe = IndexSet<Uuid>;
//              target commit id,          source commit id(s), patch id
pub type Commits = BTreeMap<Luid, TinyVec<[(TinyVec<[Luid; 2]>, Luid); 1]>>;
pub type Patches = BTreeMap<Luid, UuidSetPatch>;
pub type Heads = BTreeSet<Luid>;
pub type VersionCache = HashMap<Luid, RangeSetBlaze<Luid>>; // TODO: consider alternative data structures
pub(crate) type WorkingPatch = UuidSetPatch;
pub(crate) type WorkingState = IndexSet<Luid>;

#[derive(Default)]
pub struct TotalState {
  pub universe: Universe,
  pub commits: Commits,
  pub patches: Patches,
  pub heads: Heads,
  pub version_cache: VersionCache,
  pub(crate) working_patch: WorkingPatch,
  pub(crate) working_state: WorkingState,
}
