use i1::*;

use reedline_repl_rs::clap::{self, builder::TypedValueParser, Arg, Command};
use reedline_repl_rs::{Error, Repl};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
struct CommitUuidParser {
  state: Arc<RwLock<TotalState>>,
}

impl From<&Arc<RwLock<TotalState>>> for CommitUuidParser {
  fn from(state: &Arc<RwLock<TotalState>>) -> Self {
    Self {
      state: state.clone(),
    }
  }
}

impl TypedValueParser for CommitUuidParser {
  type Value = Uuid;
  fn parse_ref(
    &self,
    cmd: &Command,
    arg: Option<&Arg>,
    value: &std::ffi::OsStr,
  ) -> Result<Self::Value, clap::Error> {
    let inner = reedline_repl_rs::clap::builder::StringValueParser::default().parse(
      cmd,
      arg,
      value.into(),
    )?;
    let uuid = Uuid::from_base64url(&inner).map_err(|_| {
      clap::Error::raw(
        clap::error::ErrorKind::InvalidValue,
        format!("Invalid UUID: {}", inner),
      )
    })?;
    let state = self.state.read().unwrap();
    if state
      .universe
      .get_index_of(&uuid)
      .map(|luid| state.commits.contains_key(&luid))
      .unwrap_or(false)
    {
      Ok(uuid)
    } else {
      Err(clap::Error::raw(
        clap::error::ErrorKind::InvalidValue,
        format!("Unknown commit: {}", inner),
      ))
    }
  }
  fn possible_values(&self) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
    let state = self.state.read().unwrap();
    let commits: Vec<String> = state.commits().map(|id| id.as_base64url()).collect();
    Some(Box::new(
      commits.into_iter().map(clap::builder::PossibleValue::new),
    ))
  }
}

fn main() {
  let state = Arc::new(RwLock::new(TotalState::new()));
  let mut repl: Repl<_, Error> = Repl::new(state.clone())
    .with_name("uuid_set")
    .with_partial_completions(true)
    .with_command(Command::new("add"), |_, state| {
      let mut state = state.write().unwrap();
      let new_uuid = state.add();
      Ok(Some(format!(
        "Created new entity {} in working set",
        new_uuid.as_base64url()
      )))
    })
    .with_command(
      Command::new("checkout").arg(
        Arg::new("uuid")
          .required(true)
          .index(1)
          .value_parser(CommitUuidParser::from(&state)),
      ),
      |matches, state| {
        let mut state = state.write().unwrap();
        let uuid = matches.get_one::<Uuid>("uuid").unwrap();
        state.checkout(uuid);
        Ok(Some(format!("Checked out commit {}", uuid.as_base64url())))
      },
    )
    .with_command(Command::new("commit"), |_, state| {
      let mut state = state.write().unwrap();
      let (patch_id, patch) = state.commit();
      Ok(Some(format!(
        "Saved new patch {} from [{}] to {}",
        patch_id.as_base64url(),
        patch
          .source_commits
          .iter()
          .map(|id| id.as_base64url())
          .collect::<Vec<_>>()
          .join(", "),
        patch.target_commit.as_base64url(),
      )))
    })
    .with_command(Command::new("graph"), |_, state| {
      let state = state.read().unwrap();
      let graph = state.graph();
      Ok(Some(
        metro::to_string(
          graph
            .iter()
            .map(|e| e.into())
            .collect::<Vec<metro::Event>>()
            .as_slice(),
        )
        .unwrap()
        .trim_end_matches('\n')
        .to_string(),
      ))
    })
    .with_command(Command::new("load"), |_, state| {
      let mut state = state.write().unwrap();
      state.load_all_patches();
      Ok(None)
    })
    .with_command(
      Command::new("merge").arg(
        Arg::new("uuid")
          .required(true)
          .index(1)
          .value_parser(CommitUuidParser::from(&state)),
      ),
      |matches, state| {
        let mut state = state.write().unwrap();
        let uuid = matches.get_one::<Uuid>("uuid").unwrap();
        state
          .merge(uuid)
          .map(|_| Some(format!("Merged {} into working set", uuid.as_base64url())))
          .or_else(|e| Ok(Some(e.to_string())))
      },
    )
    .with_command(Command::new("count"), |_, state| {
      let state = state.read().unwrap();
      Ok(Some(format!("Entities: {:?}", state.count())))
    })
    .with_command(Command::new("commits"), |_, state| {
      let state = state.read().unwrap();
      Ok(Some(
        state
          .commits()
          .map(|uuid| format!("* {}", uuid.as_base64url()))
          .collect::<Vec<_>>()
          .join("\n"),
      ))
    })
    .with_command(Command::new("heads"), |_, state| {
      let state = state.read().unwrap();
      Ok(Some(
        state
          .heads()
          .map(|uuid| format!("* {}", uuid.as_base64url()))
          .collect::<Vec<_>>()
          .join("\n"),
      ))
    })
    .with_command(Command::new("list"), |_, state| {
      let state = state.read().unwrap();
      Ok(Some(
        state
          .list()
          .map(|uuid| format!("* {}", uuid.as_base64url()))
          .collect::<Vec<_>>()
          .join("\n"),
      ))
    });
  let _ = repl.run();
}
