// Docker primitives shared by the CLI and TUI.
//
// Provides a single streaming `spawn` primitive plus environment checks and
// platform helpers. Disk I/O is the caller's responsibility — `spawn` only
// produces output frames.

pub mod engine;
pub mod run;
pub mod types;

pub use engine::{ensure_available, ensure_available_with_compose, user_args, user_flag};
pub use run::spawn;
pub use types::{CancelToken, ContainerCommand, ContainerResult, OutputLine};
