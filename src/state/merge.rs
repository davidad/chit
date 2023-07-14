use crate::id::*;
use crate::TotalState;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub enum MergeError {
  WorkingPatchNotEmpty,
  CommitNotFound,
  DetachedHead,
  NoCommonAncestor,
}

impl std::fmt::Display for MergeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MergeError::WorkingPatchNotEmpty => write!(f, "Working patch is not empty"),
      MergeError::CommitNotFound => write!(f, "Commit not found"),
      MergeError::DetachedHead => write!(f, "Detached head"),
      MergeError::NoCommonAncestor => write!(f, "No common ancestor"),
    }
  }
}

impl TotalState {
  pub fn lca(&self, commit0: &Uuid, commit1: &Uuid) -> Option<Luid> {
    // Finding a least common ancestor is a very similar problem to rendering the commit graph, but
    // (a) we need to start from just these two commits, and only consider the commits that are
    // reachable from one of them, and (b) we need to stop at the first moment (after introducing
    // these two commits as two initial tracks) when there is only one track.
    let mut tracks: BTreeMap<Luid, usize> = BTreeMap::new();
    let mut n_tracks_total = 0;
    let mut track: usize;
    let mut queue: BTreeSet<Uuid> = BTreeSet::new();
    let mut lca: Option<Luid> = None;
    // Begin by inserting the two leaf commits to the queue.
    queue.insert(*commit0);
    queue.insert(*commit1);
    let commit0_luid = self.universe.get_index_of(commit0).unwrap();
    let commit1_luid = self.universe.get_index_of(commit1).unwrap();
    tracks.insert(commit0_luid, 0);
    tracks.insert(commit1_luid, 1);
    while let Some(uuid) = queue.pop_last() {
      let luid = self.universe.get_index_of(&uuid).unwrap();
      let reached_by = self.commits.get(&luid).unwrap();
      if let Some(existing_track) = tracks.remove(&luid) {
        track = existing_track;
      } else {
        track = n_tracks_total;
        n_tracks_total += 1;
      }
      if tracks.is_empty() {
        lca = Some(luid);
        break;
      }
      // TODO: handle distinct incoming morphisms
      let parents = reached_by.iter().flat_map(|x| x.0.iter());
      if let Some(&primary_parent) = parents.clone().next() {
        tracks.remove(&luid);
        if tracks.get(&primary_parent).is_none() {
          tracks.insert(primary_parent, track);
        }
        for &parent_luid in parents {
          let &parent_uuid = self.universe.get_index(parent_luid).unwrap();
          queue.insert(parent_uuid);
        }
      }
    }
    eprintln!(
      "LCA: {:?}",
      lca.and_then(|lca| self.universe.get_index(lca).map(|x| x.as_base64url()))
    );
    lca
  }

  pub fn merge(&mut self, commit: &Uuid) -> Result<(), MergeError> {
    let commit_luid = self
      .universe
      .get_index_of(commit)
      .ok_or(MergeError::CommitNotFound)?;
    self.merge_luid(commit_luid)
  }

  fn merge_luid(&mut self, other_commit_luid: Luid) -> Result<(), MergeError> {
    if !self.working_patch.is_empty() {
      return Err(MergeError::WorkingPatchNotEmpty);
    }
    /*
    let other_version = self.version_cache.get(&commit)
      .ok_or(()).map_err(|_| {
        let patch_spec = self.commits.get(&commit)
          .and_then(|x| x.get(0))
          .ok_or(MergeError::CommitNotFound)?;
        let patch_luid = patch_spec.1;
        process_patch(&mut self.universe, &mut self.version_cache, &mut self.commits, &mut self.heads, &self.patches, patch_luid);
        self.version_cache.get(&commit)
          .ok_or(MergeError::CommitNotFound)?
      })?;
    */
    let this_commit_uuid = *self
      .working_patch
      .source_commits
      .get(0)
      .ok_or(MergeError::DetachedHead)?;
    let _this_commit_luid = self.universe.get_index_of(&this_commit_uuid).unwrap();
    let other_commit_uuid = *self.universe.get_index(other_commit_luid).unwrap();
    let _lca = self
      .lca(&this_commit_uuid, &other_commit_uuid)
      .ok_or(MergeError::NoCommonAncestor)?;

    Ok(())
  }
}
