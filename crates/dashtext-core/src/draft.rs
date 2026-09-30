use std::fmt;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use uuid::Uuid;

/// Stable identity of a draft.
///
/// Identifiers are UUIDv7, so they sort by creation time and can be generated
/// on any device without coordination, which keeps sync possible later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DraftId(Uuid);

impl DraftId {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(Uuid::now_v7())
    }

    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for DraftId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DraftId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(f)
    }
}

impl FromStr for DraftId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

/// Milliseconds since the Unix epoch, in UTC.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
            });
        Self(millis)
    }

    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }

    /// Returns this timestamp moved back by `millis`.
    #[must_use]
    pub const fn minus_millis(self, millis: i64) -> Self {
        Self(self.0.saturating_sub(millis))
    }
}

/// Where a draft lives. Every draft is in exactly one folder.
///
/// Flagging is independent of the folder: a draft can be flagged in the
/// inbox or in the archive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Folder {
    /// The intake folder where captured drafts land until processed.
    #[default]
    Inbox,
    Archive,
    Trash,
}

impl Folder {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Archive => "archive",
            Self::Trash => "trash",
        }
    }
}

impl FromStr for Folder {
    type Err = UnknownFolder;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inbox" => Ok(Self::Inbox),
            "archive" => Ok(Self::Archive),
            "trash" => Ok(Self::Trash),
            other => Err(UnknownFolder(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown folder `{0}`")]
pub struct UnknownFolder(String);

/// A unit of captured text.
///
/// The content is the source of truth; the title and preview are derived from
/// it rather than stored, so they can never disagree with the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub(crate) id: DraftId,
    pub(crate) content: String,
    pub(crate) folder: Folder,
    pub(crate) flagged: bool,
    pub(crate) created_at: Timestamp,
    pub(crate) modified_at: Timestamp,
    pub(crate) accessed_at: Timestamp,
    pub(crate) trashed_at: Option<Timestamp>,
}

impl Draft {
    #[must_use]
    pub fn id(&self) -> DraftId {
        self.id
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[must_use]
    pub fn folder(&self) -> Folder {
        self.folder
    }

    #[must_use]
    pub fn is_flagged(&self) -> bool {
        self.flagged
    }

    #[must_use]
    pub fn modified_at(&self) -> Timestamp {
        self.modified_at
    }

    /// The first non-blank line, without Markdown heading markers.
    ///
    /// Empty when the draft has no visible text.
    #[must_use]
    pub fn title(&self) -> &str {
        title_of(&self.content)
    }

    /// The text after the title line, collapsed onto one line and truncated
    /// to at most `max_chars` characters.
    #[must_use]
    pub fn preview(&self, max_chars: usize) -> String {
        preview_of(&self.content, max_chars)
    }
}

/// Word and character counts for a piece of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStats {
    words: usize,
    characters: usize,
}

impl TextStats {
    #[must_use]
    pub fn of(text: &str) -> Self {
        Self {
            words: text.split_whitespace().count(),
            characters: text.chars().count(),
        }
    }

    #[must_use]
    pub fn words(&self) -> usize {
        self.words
    }

    #[must_use]
    pub fn characters(&self) -> usize {
        self.characters
    }
}

/// Returns the display title of `content`: its first non-blank line with any
/// leading Markdown heading markers removed.
#[must_use]
fn title_of(content: &str) -> &str {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map_or("", strip_heading_marker)
}

fn strip_heading_marker(line: &str) -> &str {
    let hashes = line.bytes().take_while(|byte| *byte == b'#').count();
    let rest = &line[hashes..];
    // `#tag` is text, not a heading: CommonMark requires whitespace after the markers.
    if hashes > 0 && hashes <= 6 && (rest.is_empty() || rest.starts_with([' ', '\t'])) {
        rest.trim()
    } else {
        line
    }
}

fn preview_of(content: &str, max_chars: usize) -> String {
    let mut lines = content
        .lines()
        .map(str::trim)
        .skip_while(|line| line.is_empty());
    // Skip the title line.
    lines.next();

    let mut preview = String::new();
    let mut length = 0;
    for word in lines.flat_map(str::split_whitespace) {
        let word_length = word.chars().count();
        let separator = usize::from(length > 0);
        if length + separator + word_length > max_chars {
            if length == 0 {
                // A single overlong word: cut it rather than show nothing.
                preview.extend(word.chars().take(max_chars.saturating_sub(1)));
            }
            preview.push('…');
            break;
        }
        if separator > 0 {
            preview.push(' ');
        }
        preview.push_str(word);
        length += separator + word_length;
    }
    preview
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_first_non_blank_line() {
        assert_eq!(title_of("\n\n  Groceries  \nmilk"), "Groceries");
    }

    #[test]
    fn title_strips_heading_markers() {
        assert_eq!(title_of("## Meeting notes\nbody"), "Meeting notes");
        assert_eq!(title_of("#\nbody"), "");
    }

    #[test]
    fn title_keeps_hashtags() {
        assert_eq!(title_of("#idea for later"), "#idea for later");
        assert_eq!(title_of("####### seven"), "####### seven");
    }

    #[test]
    fn title_of_blank_content_is_empty() {
        assert_eq!(title_of(""), "");
        assert_eq!(title_of(" \n\t\n"), "");
    }

    #[test]
    fn preview_skips_title_and_collapses_whitespace() {
        assert_eq!(
            preview_of("Title\n\nfirst  line\nsecond", 80),
            "first line second"
        );
    }

    #[test]
    fn preview_truncates_with_ellipsis() {
        assert_eq!(preview_of("Title\none two three four", 9), "one two…");
        assert_eq!(preview_of("Title\nsupercalifragilistic", 6), "super…");
    }

    #[test]
    fn preview_of_single_line_is_empty() {
        assert_eq!(preview_of("Only a title", 80), "");
    }

    #[test]
    fn stats_count_words_and_characters() {
        let stats = TextStats::of("héllo world\nagain");
        assert_eq!(stats.words(), 3);
        assert_eq!(stats.characters(), 17);
        assert_eq!(TextStats::of(""), TextStats::default());
    }

    #[test]
    fn folder_round_trips_through_str() {
        for folder in [Folder::Inbox, Folder::Archive, Folder::Trash] {
            assert_eq!(folder.as_str().parse::<Folder>().ok(), Some(folder));
        }
        assert!("elsewhere".parse::<Folder>().is_err());
    }

    #[test]
    fn draft_ids_parse_their_display_form() {
        let id = DraftId::new();
        assert_eq!(id.to_string().parse::<DraftId>().ok(), Some(id));
    }
}
