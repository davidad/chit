use base64ct::{Base64UrlUnpadded, Encoding};
use indexmap::{IndexSet};
use memmap2::Mmap;
use range_set_blaze::RangeSetBlaze;
use rkyv::{Archive, Deserialize, Serialize, ser::{Serializer, serializers::{WriteSerializer, CompositeSerializer, FallbackScratch, AllocScratch, HeapScratch}}, validation::validators::{check_archived_root}};
use std::{collections::{BTreeSet, BTreeMap, HashSet, HashMap}, fs::{self, File}, cell::RefCell, thread_local, path::PathBuf};
use tinyvec::TinyVec;
use uuid::Uuid;

thread_local! {
  static RKYV_SCRATCH : RefCell<FallbackScratch<HeapScratch<{1 << 27}>, AllocScratch>> = RefCell::new(FallbackScratch::new(HeapScratch::new(), AllocScratch::new()));
}

pub trait AsBase64Url {
  fn as_base64url(&self) -> String;
}

impl AsBase64Url for Uuid {
  fn as_base64url(&self) -> String {
    Base64UrlUnpadded::encode_string(self.as_bytes())
  }
}

pub trait FromBase64Url {
  fn from_base64url(s: &str) -> Result<Self, String> where Self: Sized;
}

impl FromBase64Url for Uuid {
  fn from_base64url(s: &str) -> Result<Self, String> {
    let bytes = Base64UrlUnpadded::decode_vec(s).map_err(|e| format!("Error decoding UUID in parsing base64url: {}", e))?;
    Ok(Uuid::from_slice(bytes.as_slice()).map_err(|e| format!("Error decoding UUID format: {}", e))?)
  }
}

#[derive(Default, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct UuidSetPatch {
  pub target_commit: Uuid,
  pub source_commits: TinyVec<[Uuid; 2]>,
  pub deletions: HashSet<Uuid>,
  pub additions: HashSet<Uuid>,
  pub merges: HashMap<Uuid, Uuid>,
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

#[derive(Default, Clone)]
pub struct UuidSet {
  pub set: HashSet<Uuid>,
}

impl UuidSet {
  pub fn new() -> Self {
    Self {
      set: HashSet::new(),
    }
  }
} // impl UuidSet

type Luid = usize;
type Universe = IndexSet<Uuid>;
  //         target commit id,          source commit id(s), patch id
type Commits = BTreeMap<Luid, TinyVec<[(TinyVec<[Luid; 2]>, Luid); 1]>>;
type Patches = BTreeMap<Luid, UuidSetPatch>;
type Heads = BTreeSet<Luid>;
type VersionCache = HashMap<Luid, RangeSetBlaze<Luid>>; // TODO: consider alternative data structures
type WorkingPatch = UuidSetPatch;
type WorkingState = IndexSet<Luid>;

#[derive(Default)]
pub struct TotalState {
  pub universe : Universe,
  pub commits : Commits,
  pub patches : Patches,
  pub heads : Heads,
  pub version_cache : VersionCache,
  working_patch: WorkingPatch,
  working_state: WorkingState,
}

// TODO:
// * implement "heads"
// * implement deletions
// * implement merges
// * implement namings: each naming associates a fully-qualified name to a Uuid and also has a Uuid of its own, and an optional message.
// * implement authenticated states: a SHA3 hash of a naming concatenated with a canonicalized working state
// * Add a "revert" command that reverts the working state to the last committed state
// * Add a "revert" command that reverts the working state to a specific commit
// * Add a "merge" command that merges the working state with a specific commit

fn process_patch(universe: &mut Universe, version_cache: &mut VersionCache, commits: &mut Commits, heads: &mut Heads, patches: &Patches, patch_luid: Luid) {
  let patch = patches.get(&patch_luid).unwrap();
  let mut version : IndexSet<Luid> = IndexSet::new();
  for source_commit in patch.source_commits.iter() {
    let source_commit_luid = universe.get_index_of(source_commit).unwrap();
    let mut source_version = version_cache.get(&source_commit_luid);
    if source_version.is_none() {
      eprintln!("Info: patch {:?} depends on {:?} which has not been processed yet. Processing it now.",
        universe.get_index(patch_luid).unwrap().as_base64url(),
        universe.get_index(source_commit_luid).unwrap().as_base64url());
      process_patch(universe, version_cache, commits, heads, patches, commits.get(&source_commit_luid).unwrap().get(0).unwrap().1);
      source_version = Some(version_cache.get(&source_commit_luid).unwrap());
    }
    heads.remove(&source_commit_luid);
    version.extend(source_version.unwrap().iter());
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
  version.extend(patch.additions.iter().map(|uuid| universe.get_index_of(uuid).unwrap()));
  let target_commit_luid = universe.insert_full(patch.target_commit).0;
  heads.insert(target_commit_luid);
  version_cache.insert(target_commit_luid, version.into_iter().collect());
}

#[derive(Clone, Debug)]
pub enum GraphEvent {
  StartTrack(usize),
  StopTrack(usize),
  Station(usize, String),
  SplitTrack(usize, usize),
  JoinTrack(usize, usize),
  NoEvent,
}
use GraphEvent::*;

impl<'a> From<&'a GraphEvent> for metro::Event<'a> {
  fn from(event: &'a GraphEvent) -> Self {
    match event {
      GraphEvent::StartTrack(track_id) => metro::Event::StartTrack(*track_id),
      GraphEvent::StopTrack(track_id) => metro::Event::StopTrack(*track_id),
      GraphEvent::Station(track_id, station_name) => metro::Event::Station(*track_id, station_name.as_str()),
      GraphEvent::SplitTrack(track_id, new_track_id) => metro::Event::SplitTrack(*track_id, *new_track_id),
      GraphEvent::JoinTrack(track_id, new_track_id) => metro::Event::JoinTrack(*track_id, *new_track_id),
      GraphEvent::NoEvent => metro::Event::NoEvent,
    }
  }
}

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
    self.commits.iter().map(|(luid, _)| self.universe.get_index(*luid).unwrap())
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
    self.working_patch.source_commits.push(*self.universe.get_index(commit).unwrap());
    let version = self.version_cache.get(&commit)?;
    self.working_state = version.iter().collect();
    Some(())
  }
  
  pub fn graph(&self) -> Vec<GraphEvent>{
    let mut graph = Vec::with_capacity(self.commits.len());
    let mut tracks : HashMap<Luid, usize> = HashMap::new();
    let mut n_tracks : usize = 0;
    let mut track : usize;
    for (commit_luid, reachable_by) in self.commits.iter().rev() {
      let reached_by = reachable_by.get(0).unwrap();
      if let Some(&existing_track) = tracks.get(commit_luid) {
        track = existing_track;
      } else {
        track = n_tracks;
        n_tracks += 1;
        tracks.insert(*commit_luid, track);
        graph.push(NoEvent);
        graph.push(StartTrack(track));
      }
      graph.push(Station(track,
        format!("{} <- {}",
          self.universe.get_index(*commit_luid).unwrap().as_base64url(),
          self.universe.get_index(reached_by.1).unwrap().as_base64url())));
      if reached_by.0.is_empty() {
        graph.push(NoEvent);
        graph.push(StopTrack(track));
        graph.push(NoEvent);
      } else {
        for &parent in reached_by.0.iter().skip(1) {
          let parent_track = tracks.len();
          if let Some(existing_track) = tracks.insert(parent, parent_track) {
            graph.push(SplitTrack(track, parent_track));
            graph.push(JoinTrack(parent_track, existing_track));
            tracks.insert(parent, existing_track);
          } else {
            graph.push(SplitTrack(track, parent_track));
          }
        }
        let primary_parent = reached_by.0[0];
        if let Some(&existing_track) = tracks.get(&primary_parent) {
          graph.push(JoinTrack(track, existing_track));
          tracks.insert(primary_parent, existing_track);
        } else {
          tracks.insert(primary_parent, track);
        }
      }
    }
    graph
  }

  pub fn heads(&self) -> impl Iterator<Item = &Uuid> {
    self.heads.iter().map(|luid| self.universe.get_index(*luid).unwrap())
  }

  pub fn load_all_patches(&mut self) {
    let patch_dir = Self::get_patch_dir();
    fs::create_dir_all(&patch_dir).unwrap();
    let patch_files = std::fs::read_dir(patch_dir).unwrap()
      .map(|entry| entry.unwrap().path())
      .collect::<BTreeSet<_>>();
    let len = patch_files.len();
    for (i, path) in patch_files.iter().enumerate() {
      let f = File::open(path).unwrap();
      let mmap = unsafe { Mmap::map(&f).unwrap() };
      let patch = check_archived_root::<UuidSetPatch>(mmap.as_ref()).unwrap();

      eprintln!("Loading patch {}/{}: {:?}", i+1, len, path.file_name().unwrap().to_str().unwrap().strip_prefix("patch_").unwrap());
      let patch_uuid = Uuid::from_base64url(
        path.file_name().unwrap()
          .to_str().unwrap()
          .strip_prefix("patch_").unwrap()
        ).unwrap();
      // (note: this deserialization may perform unnecessary copies, but is memory-safe)
      self.index_patch(patch_uuid, patch.deserialize(&mut rkyv::Infallible).unwrap());
    }
    let patch_luids : Vec<Luid> = self.patches.keys().copied().collect(); // TODO: this is a temporary hack to avoid borrowing self
    for patch_luid in patch_luids {
      process_patch(&mut self.universe, &mut self.version_cache, &mut self.commits, &mut self.heads, &self.patches, patch_luid);
    }

    if let Some(&head) = self.heads.last() {
      self.checkout_luid(head);
    }
  }

  fn index_patch(&mut self, patch_uuid: Uuid, patch: UuidSetPatch) -> Luid {
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
    self.commits.insert(target_commit_luid, TinyVec::from([(patch_ref.source_commits.iter().map(|uuid| self.universe.insert_full(*uuid).0).collect(), patch_luid)]));

    patch_luid
  }

  pub fn add(&mut self) -> Uuid {
    let uuid = Uuid::now_v7();
    self.working_state.insert(self.universe.insert_full(uuid).0);
    self.working_patch.additions.insert(uuid);
    uuid
  }

  pub fn commit(&mut self) -> (Uuid, &UuidSetPatch) {
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
    RKYV_SCRATCH.with(|scratch|  {
      let scratch_inner = scratch.replace(
        FallbackScratch::new(HeapScratch::new(), AllocScratch::new()));
      let mut serializer: CompositeSerializer<WriteSerializer<_>, FallbackScratch<_,_>, _> =
        CompositeSerializer::new(
          WriteSerializer::new(&mut file),
          scratch_inner,
          rkyv::Infallible::default());

      serializer.serialize_value(&self.working_patch).unwrap();

      serializer.into_components().1 // return borrowed scratch space
    });

    let written_patch = std::mem::replace(&mut self.working_patch, UuidSetPatch::default());
    self.working_patch.clear();
    let new_patch_luid = self.index_patch(new_patch_id, written_patch.into());
    process_patch(&mut self.universe, &mut self.version_cache, &mut self.commits, &mut self.heads, &self.patches,
      new_patch_luid);
    self.working_patch.source_commits.push(new_commit_id);
    self.working_patch.target_commit = Default::default();

    (new_patch_id, self.patches.get(&new_patch_luid).unwrap())
  }

  pub fn count(&self) -> usize {
    self.working_state.len()
  }

  pub fn list(&self) -> impl Iterator<Item = &Uuid> {
    self.working_state.iter().map(move |luid| self.universe.get_index(*luid).unwrap())
  }
}