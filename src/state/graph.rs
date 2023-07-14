use crate::id::*;
use crate::TotalState;
use std::collections::HashMap;

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
      GraphEvent::Station(track_id, station_name) => {
        metro::Event::Station(*track_id, station_name.as_str())
      }
      GraphEvent::SplitTrack(track_id, new_track_id) => {
        metro::Event::SplitTrack(*track_id, *new_track_id)
      }
      GraphEvent::JoinTrack(track_id, new_track_id) => {
        metro::Event::JoinTrack(*track_id, *new_track_id)
      }
      GraphEvent::NoEvent => metro::Event::NoEvent,
    }
  }
}

impl TotalState {
  pub fn graph(&self) -> Vec<GraphEvent> {
    let mut graph = Vec::with_capacity(self.commits.len());
    let mut tracks: HashMap<Luid, usize> = HashMap::new();
    let mut n_tracks_total: usize = 0;
    let mut track: usize;
    for (commit_luid, reachable_by) in self.commits.iter().rev() {
      let reached_by = reachable_by.get(0).unwrap();
      if let Some(&existing_track) = tracks.get(commit_luid) {
        track = existing_track;
      } else {
        track = n_tracks_total;
        n_tracks_total += 1;
        tracks.insert(*commit_luid, track);
        graph.push(NoEvent);
        graph.push(StartTrack(track));
      }
      graph.push(Station(
        track,
        format!(
          "{} <- {}",
          self
            .universe
            .get_index(*commit_luid)
            .unwrap()
            .as_base64url(),
          self
            .universe
            .get_index(reached_by.1)
            .unwrap()
            .as_base64url()
        ),
      ));
      if reached_by.0.is_empty() {
        graph.push(NoEvent);
        graph.push(StopTrack(track));
        graph.push(NoEvent);
      } else {
        for &parent in reached_by.0.iter().skip(1) {
          let parent_track = n_tracks_total;
          n_tracks_total += 1;
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
        } else {
          tracks.insert(primary_parent, track);
        }
      }
    }
    graph
  }
}
