mod metric_collector;
#[cfg(all(feature = "test-fixture", feature = "web-app", feature = "cli"))]
pub mod playbook;
mod verbosity;

pub use metric_collector::{install_collector, CollectorHandle};
pub use verbosity::Verbosity;
