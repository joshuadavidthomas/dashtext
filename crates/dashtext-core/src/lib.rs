//! The Dashtext draft library: the domain model and its SQLite storage.
//!
//! This crate has no UI dependencies so the desktop app, the command line and
//! future front ends (and an actions runtime) can share one library format.

mod draft;
mod query;
mod store;
mod workspace;

pub use draft::Draft;
pub use draft::DraftId;
pub use draft::Folder;
pub use draft::TextStats;
pub use draft::Timestamp;
pub use query::Scope;
pub use query::SearchQuery;
pub use query::Sort;
pub use query::SortDirection;
pub use query::SortKey;
pub use store::Result;
pub use store::ScopeCounts;
pub use store::Store;
pub use store::StoreError;
pub use workspace::Workspace;
pub use workspace::WorkspaceId;
