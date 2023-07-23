use std::path::PathBuf;

mod context;
pub use context::Context;
mod id;
pub use id::{AsBase64Url, FromBase64Url, Luid, Slid, Uuid, Vlid};
mod patch;
pub use patch::UuidSetPatch;
mod state;
pub use state::TotalState;
mod version;
pub use version::Version;

impl TotalState {
  pub fn new() -> Self {
    let mut state = Self::default();
    state.load_all_patches();
    state
  }

  pub fn get_patch_dir() -> PathBuf {
    let mut patch_dir = PathBuf::new();
    patch_dir.push("patches/");
    patch_dir
  }

  pub fn commits(&self) -> impl Iterator<Item = &Uuid> {
    self
      .commits
      .keys()
      .map(|luid| self.universe.get_index(*luid).unwrap())
  }

  pub fn checkout(&mut self, commit: &Uuid) -> Option<()> {
    let commit_luid = self.universe.get_index_of(commit)?;
    self.checkout_luid(commit_luid)
  }

  pub fn checkout_luid(&mut self, commit: Luid) -> Option<()> {
    if !self.working_patch.is_empty() {
      eprintln!("Error: checkout while working patch is not empty. Commit before checking out.");
    }
    self.working_patch.clear();
    self
      .working_patch
      .source_commits
      .push(*self.universe.get_index(commit).unwrap());
    let version = self.version_cache.get(&commit)?;
    self.working_state = version
      .version_universe
      .iter()
      .map(|x| x as usize)
      .collect();
    Some(())
  }

  pub fn heads(&self) -> impl Iterator<Item = &Uuid> {
    self
      .heads
      .iter()
      .map(|luid| self.universe.get_index(*luid).unwrap())
  }

  pub fn add(&mut self) -> Uuid {
    let uuid = Uuid::now_v7();
    self.working_state.insert(self.universe.insert_full(uuid).0);
    self.working_patch.universe_patch.additions.insert(uuid);
    uuid
  }

  pub fn count(&self) -> usize {
    self.working_state.len()
  }

  pub fn list(&self) -> impl Iterator<Item = &Uuid> {
    self
      .working_state
      .iter()
      .map(move |luid| self.universe.get_index(*luid).unwrap())
  }
}
