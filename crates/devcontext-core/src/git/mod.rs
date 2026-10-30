pub mod adapter;
pub mod model;
pub mod timeline;

pub use adapter::GitAdapter;
pub use model::GitCommitInfo;
pub use timeline::{TimelineEvent, TimelineEventType, TimelineFilter, TimelineService};
