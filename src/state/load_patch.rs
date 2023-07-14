use crate::id::*;
use crate::patch::*;
use crate::state::*;
use memmap2::Mmap;
use rkyv::{check_archived_root, Deserialize};
use std::collections::BTreeSet;
use std::fs::{self, File};
use tinyvec::TinyVec;

impl TotalState {
  pub fn load_all_patches(&mut self) {
    let patch_dir = Self::get_patch_dir();
    fs::create_dir_all(&patch_dir).unwrap();
    let patch_files = std::fs::read_dir(patch_dir)
      .unwrap()
      .map(|entry| entry.unwrap().path())
      .collect::<BTreeSet<_>>();
    let len = patch_files.len();
    for (i, path) in patch_files.iter().enumerate() {
      let f = File::open(path).unwrap();
      let mmap = unsafe { Mmap::map(&f).unwrap() };
      let patch = check_archived_root::<UuidSetPatch>(mmap.as_ref()).unwrap();
      let patch_uuid_str = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("patch_")
        .unwrap();
      eprintln!("Loading patch {}/{}: {:?}", i + 1, len, patch_uuid_str);
      let patch_uuid = Uuid::from_base64url(patch_uuid_str).unwrap();
      // (note: this deserialization may perform unnecessary copies, but is memory-safe)
      self.index_patch(
        patch_uuid,
        patch.deserialize(&mut rkyv::Infallible).unwrap(),
      );
    }
    let patch_luids: Vec<(Luid, Luid)> = self
      .patches
      .iter()
      .map(|(k, v)| (*k, self.universe.get_index_of(&v.target_commit).unwrap()))
      .collect(); // TODO: this is a temporary hack to avoid borrowing self
    for (patch_luid, target_commit_luid) in patch_luids {
      if !self.version_cache.contains_key(&target_commit_luid) {
        process_patch(
          &mut self.universe,
          &mut self.version_cache,
          &mut self.commits,
          &mut self.heads,
          &self.patches,
          patch_luid,
        );
      }
    }
    if let Some(&head) = self.heads.last() {
      self.checkout_luid(head);
    }
  }

  pub(in crate::state) fn index_patch(&mut self, patch_uuid: Uuid, patch: UuidSetPatch) -> Luid {
    // add patch to universe
    let patch_luid = self.universe.insert_full(patch_uuid).0;
    // add patch to patches
    self.patches.insert(patch_luid, patch);
    let patch_ref = self.patches.get(&patch_luid).unwrap();
    // add patch contents to universe
    patch_ref.additions.iter().for_each(|uuid| {
      self.universe.insert(*uuid);
    });
    // add target commit to universe
    let target_commit_luid = self.universe.insert_full(patch_ref.target_commit).0;
    // add patch to commits
    self.commits.insert(
      target_commit_luid,
      TinyVec::from([(
        patch_ref
          .source_commits
          .iter()
          .map(|uuid| self.universe.insert_full(*uuid).0)
          .collect(),
        patch_luid,
      )]),
    );
    patch_luid
  }
}
