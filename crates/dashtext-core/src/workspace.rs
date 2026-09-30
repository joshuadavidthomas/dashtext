use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::query::Scope;
use crate::query::Sort;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
    #[must_use]
    fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(f)
    }
}

impl FromStr for WorkspaceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

/// A saved view of the draft library.
///
/// A workspace remembers how the library is presented: the selected scope
/// and the sort order. Filters such as tags and saved searches belong here
/// too as they are added, so switching workspaces switches the whole view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    view: WorkspaceView,
}

/// The serialized, evolvable part of a workspace.
///
/// Stored as JSON so new view settings can be added without a schema
/// migration; unknown or missing fields fall back to their defaults.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct WorkspaceView {
    scope: Scope,
    sort: Sort,
}

impl Workspace {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: WorkspaceId::new(),
            name: name.into(),
            view: WorkspaceView::default(),
        }
    }

    pub(crate) fn from_parts(id: WorkspaceId, name: String, view: WorkspaceView) -> Self {
        Self { id, name, view }
    }

    #[must_use]
    pub(crate) fn id(&self) -> WorkspaceId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn scope(&self) -> Scope {
        self.view.scope
    }

    #[must_use]
    pub fn sort(&self) -> Sort {
        self.view.sort
    }

    pub(crate) fn view(&self) -> &WorkspaceView {
        &self.view
    }

    pub fn set_scope(&mut self, scope: Scope) {
        self.view.scope = scope;
    }
}
