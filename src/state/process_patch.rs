use crate::id::*;
use crate::state::*;
use crate::version::*;
use indexmap::IndexSet;

pub fn process_patch(
  universe: &mut Universe,
  version_cache: &mut VersionCache,
  commits: &mut Commits,
  heads: &mut Heads,
  patches: &Patches,
  patch_luid: Luid,
) {
  let patch = patches.get(&patch_luid).unwrap();
  let mut version: IndexSet<Luid> = IndexSet::new();
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
    version.extend(source_version.unwrap().local_universe.iter());
  }
  version.retain(|luid| {
    let uuid = universe.get_index(*luid).unwrap();
    if patch.deletions.contains(uuid) {
      false
    } else if let Some(merged_into) = patch.merges.get(uuid) {
      uuid == merged_into
    } else {
      true
    }
  });
  version.extend(
    patch
      .additions
      .iter()
      .map(|uuid| universe.get_index_of(uuid).unwrap()),
  );
  let target_commit_luid = universe.insert_full(patch.target_commit).0;
  heads.insert(target_commit_luid);
  version_cache.insert(
    target_commit_luid,
    Version {
      local_universe: version.into_iter().collect(),
    },
  );
}
