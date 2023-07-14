mod total_state;
pub use total_state::*;
mod commit;
pub use commit::*;
mod graph;
pub use graph::*;
mod load_patch;
pub use load_patch::*;
mod merge;
pub use merge::*;
mod process_patch;
pub use process_patch::process_patch;

// TODO:
// * implement merges
// * implement deletions
// * implement conflict checks
// * implement namings: each naming associates a fully-qualified name to a Uuid and also has a Uuid of its own, and an optional message.
// * implement authenticated states: a SHA3 hash of a naming concatenated with a canonicalized working state
// * Add a "revert" command that reverts the working state to the last committed state
// * Add a "revert" command that reverts the working state to a specific commit
// * Add a "merge" command that merges the working state with a specific commit
