use crate::id::*;
use crate::patch::*;
use crate::state::*;
use crate::version::*;
use roaring::RoaringTreemap;
use std::ops::*;

pub fn process_patch(
  universe: &mut Universe,
  version_cache: &mut VersionCache,
  commits: &mut Commits,
  heads: &mut Heads,
  patches: &Patches,
  patch_luid: Luid,
) {
  let patch = patches.get(&patch_luid).unwrap();
  let mut version_universe: RoaringTreemap = RoaringTreemap::new();
  for source_commit in patch.source_commits.iter() {
    let source_commit_luid = universe.get_index_of(source_commit).unwrap();
    let mut source_version = version_cache.get(&source_commit_luid);
    if source_version.is_none() {
      eprintln!(
        "Info: patch {:?} depends on {:?} which has not been processed yet. Processing it now.",
        universe.get_index(patch_luid).unwrap().as_base64url(),
        universe
          .get_index(source_commit_luid)
          .unwrap()
          .as_base64url()
      );
      process_patch(
        universe,
        version_cache,
        commits,
        heads,
        patches,
        commits.get(&source_commit_luid).unwrap().get(0).unwrap().1,
      );
      source_version = Some(version_cache.get(&source_commit_luid).unwrap());
    }
    heads.remove(&source_commit_luid);
    version_universe.bitor_assign(&source_version.unwrap().version_universe);
  }
  {
    // Handle universe patch
    let universe_patch = &patch.universe_patch;
    universe_patch.deletions.iter().for_each(|uuid| {
      version_universe.remove(universe.get_index_of(uuid).unwrap() as u64);
    });
    universe_patch
      .merges
      .iter()
      .for_each(|(uuid, merged_into)| {
        if uuid != merged_into {
          version_universe.remove(universe.get_index_of(uuid).unwrap() as u64);
        }
      });
    version_universe.extend(
      universe_patch
        .additions
        .iter()
        .map(|uuid| universe.get_index_of(uuid).unwrap() as u64),
    );
  }
  {
    // TODO: Handle addition kinds
    let addition_kinds = &patch.addition_kinds;
    for (_i, kind) in addition_kinds.iter().enumerate() {
      match kind {
        AdditionKind::NewSort => {}
        AdditionKind::NewEntity(_sort_uuid) => {}
      }
    }
  }
  { // TODO: Handle context patch
  }
  let target_commit_luid = universe.insert_full(patch.target_commit).0;
  heads.insert(target_commit_luid);
  version_cache.insert(
    target_commit_luid,
    Version {
      version_universe: version_universe.into_iter().collect(),
      s0: Default::default(),
      s0i: Default::default(),
      ctx: Default::default(),
    },
  );
}
