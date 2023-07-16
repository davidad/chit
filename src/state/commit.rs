use crate::id::*;
use crate::patch::*;
use crate::state::*;
use rkyv::ser::{
  serializers::{AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch, WriteSerializer},
  Serializer,
};
use std::{
  cell::RefCell,
  fs::{self, File},
  thread_local,
};

thread_local! {
  static RKYV_SCRATCH : RefCell<FallbackScratch<HeapScratch<{1 << 27}>, AllocScratch>> = RefCell::new(FallbackScratch::new(HeapScratch::new(), AllocScratch::new()));
}

impl TotalState {
  pub fn commit(&mut self) -> (Uuid, &Patch) {
    // make a new UUID for the patch
    let new_patch_id = Uuid::now_v7();
    // make a new UUID for the commit
    let new_commit_id = Uuid::now_v7();
    // set the target_commit_id of the patch to the new_commit_id
    self.working_patch.target_commit = new_commit_id;

    // convert the patch UUID to a filename-friendly string
    let new_patch_id_str = new_patch_id.as_base64url();

    // open the file for writing
    let patch_dir = Self::get_patch_dir();
    fs::create_dir_all(&patch_dir).unwrap();
    let mut file = File::create(patch_dir.join("patch_".to_string() + &new_patch_id_str)).unwrap();

    // Serialize the patch to the file using rkyv::ser::serializers::WriteSerializer
    RKYV_SCRATCH.with(|scratch| {
      let scratch_inner = scratch.replace(FallbackScratch::new(
        HeapScratch::new(),
        AllocScratch::new(),
      ));
      let mut serializer: CompositeSerializer<WriteSerializer<_>, FallbackScratch<_, _>, _> =
        CompositeSerializer::new(
          WriteSerializer::new(&mut file),
          scratch_inner,
          rkyv::Infallible,
        );

      serializer.serialize_value(&self.working_patch).unwrap();

      serializer.into_components().1 // return borrowed scratch space
    });

    let written_patch = std::mem::take(&mut self.working_patch);
    self.working_patch.clear();
    let new_patch_luid = self.index_patch(new_patch_id, written_patch);
    process_patch(
      &mut self.universe,
      &mut self.version_cache,
      &mut self.commits,
      &mut self.heads,
      &self.patches,
      new_patch_luid,
    );
    self.working_patch.source_commits.push(new_commit_id);
    self.working_patch.target_commit = Default::default();

    (new_patch_id, self.patches.get(&new_patch_luid).unwrap())
  }
}
