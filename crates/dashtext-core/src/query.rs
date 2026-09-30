use std::cmp::Ordering;

use serde::Deserialize;
use serde::Serialize;

use crate::draft::Draft;
use crate::draft::Folder;

/// A view onto the draft library, mirroring the list tabs in Drafts.
///
/// Folders hold drafts; `Flagged` and `All` are views across folders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Inbox,
    /// Flagged drafts outside the trash.
    Flagged,
    Archive,
    /// Every draft outside the trash.
    All,
    Trash,
}

impl Scope {
    pub const ALL: [Self; 5] = [
        Self::Inbox,
        Self::Flagged,
        Self::Archive,
        Self::All,
        Self::Trash,
    ];

    #[must_use]
    pub fn contains(self, draft: &Draft) -> bool {
        match self {
            Self::Inbox => draft.folder() == Folder::Inbox,
            Self::Flagged => draft.is_flagged() && draft.folder() != Folder::Trash,
            Self::Archive => draft.folder() == Folder::Archive,
            Self::All => draft.folder() != Folder::Trash,
            Self::Trash => draft.folder() == Folder::Trash,
        }
    }
}

/// Which timestamp orders a draft list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    Created,
    #[default]
    Modified,
    Accessed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sort {
    pub(crate) key: SortKey,
    pub(crate) direction: SortDirection,
}

impl Sort {
    /// Orders two drafts the way [`crate::Store::drafts`] does.
    #[must_use]
    pub fn compare(self, a: &Draft, b: &Draft) -> Ordering {
        let key = |draft: &Draft| match self.key {
            SortKey::Created => draft.created_at,
            SortKey::Modified => draft.modified_at,
            SortKey::Accessed => draft.accessed_at,
        };
        let ascending = key(a).cmp(&key(b)).then_with(|| a.id().cmp(&b.id()));
        match self.direction {
            SortDirection::Ascending => ascending,
            SortDirection::Descending => ascending.reverse(),
        }
    }

    /// The SQL `ORDER BY` clause for this sort. Built only from fixed
    /// fragments, never from user input.
    pub(crate) fn order_by(self) -> &'static str {
        match (self.key, self.direction) {
            (SortKey::Created, SortDirection::Ascending) => "created_at ASC, id ASC",
            (SortKey::Created, SortDirection::Descending) => "created_at DESC, id DESC",
            (SortKey::Modified, SortDirection::Ascending) => "modified_at ASC, id ASC",
            (SortKey::Modified, SortDirection::Descending) => "modified_at DESC, id DESC",
            (SortKey::Accessed, SortDirection::Ascending) => "accessed_at ASC, id ASC",
            (SortKey::Accessed, SortDirection::Descending) => "accessed_at DESC, id DESC",
        }
    }
}

/// A parsed search string.
///
/// Words must all appear in the draft, `"quoted phrases"` must appear
/// verbatim, and a leading `-` excludes drafts containing the term. Matching
/// ignores case.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchQuery {
    include: Vec<String>,
    exclude: Vec<String>,
}

impl SearchQuery {
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let mut query = Self::default();
        let mut chars = input.chars().peekable();

        loop {
            while chars.next_if(|c| c.is_whitespace()).is_some() {}
            let Some(&first) = chars.peek() else {
                break;
            };

            let negated = first == '-';
            if negated {
                chars.next();
            }

            let mut term = String::new();
            if chars.next_if_eq(&'"').is_some() {
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                    term.push(c);
                }
            } else {
                while let Some(c) = chars.next_if(|c| !c.is_whitespace()) {
                    term.push(c);
                }
            }

            let term = term.trim().to_lowercase();
            if term.is_empty() {
                continue;
            }
            if negated {
                query.exclude.push(term);
            } else {
                query.include.push(term);
            }
        }

        query
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    #[cfg(test)]
    fn matches(&self, text: &str) -> bool {
        self.is_empty() || self.matches_lowercase(&text.to_lowercase())
    }

    /// Whether lowercased `text` matches. Callers fold case once and reuse
    /// the result across searches.
    #[must_use]
    pub fn matches_lowercase(&self, text: &str) -> bool {
        self.include.iter().all(|term| text.contains(term.as_str()))
            && !self.exclude.iter().any(|term| text.contains(term.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything() {
        let query = SearchQuery::parse("   ");
        assert!(query.is_empty());
        assert!(query.matches("anything"));
    }

    #[test]
    fn words_must_all_match_ignoring_case() {
        let query = SearchQuery::parse("Milk eggs");
        assert!(query.matches("buy eggs and MILK"));
        assert!(!query.matches("buy milk"));
    }

    #[test]
    fn phrases_match_verbatim() {
        let query = SearchQuery::parse("\"call mom\"");
        assert!(query.matches("remember to call mom"));
        assert!(!query.matches("mom will call"));
    }

    #[test]
    fn minus_excludes_terms_and_phrases() {
        let query = SearchQuery::parse("todo -done -\"on hold\"");
        assert!(query.matches("todo: write docs"));
        assert!(!query.matches("todo: done"));
        assert!(!query.matches("todo: on hold"));
    }

    #[test]
    fn unterminated_phrase_runs_to_end() {
        assert_eq!(
            SearchQuery::parse("\"open ended"),
            SearchQuery {
                include: vec!["open ended".into()],
                exclude: vec![],
            }
        );
    }

    #[test]
    fn lone_minus_is_ignored() {
        assert!(SearchQuery::parse("- ").is_empty());
    }
}
