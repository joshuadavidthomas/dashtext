use std::cmp::Ordering;

use dashtext_core::Draft;
use dashtext_core::DraftId;
use dashtext_core::Scope;
use dashtext_core::SearchQuery;
use dashtext_core::Sort;
use gpui_kit::App;
use gpui_kit::Context;
use gpui_kit::ElementId;
use gpui_kit::Entity;
use gpui_kit::FontWeight;
use gpui_kit::IntoElement;
use gpui_kit::ParentElement as _;
use gpui_kit::Render;
use gpui_kit::SharedString;
use gpui_kit::Styled as _;
use gpui_kit::Task;
use gpui_kit::Window;
use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Icon;
use gpui_kit::component::IndexPath;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::h_flex;
use gpui_kit::component::list::List;
use gpui_kit::component::list::ListDelegate;
use gpui_kit::component::list::ListItem;
use gpui_kit::component::list::ListState;
use gpui_kit::component::v_flex;
use gpui_kit::div;
use gpui_kit::prelude::FluentBuilder as _;

use crate::time_format;

/// Longest preview shown under a draft's title, in characters.
const PREVIEW_CHARS: usize = 160;

/// Presents the drafts of one scope, filtered by the search field.
///
/// The list owns presentation and search matching; the drafts window owns
/// which scope is loaded and what happens when a row is chosen.
pub struct DraftList {
    scope: Scope,
    rows: Vec<Row>,
    query: SearchQuery,
    /// Indices into `rows` that match `query`, in display order.
    matches: Vec<usize>,
}

/// A draft with everything its row shows, derived once when the list is
/// loaded rather than on every frame (rows re-render on hover and scroll).
struct Row {
    draft: Draft,
    title: SharedString,
    preview: SharedString,
    date: SharedString,
    /// Lowercased content, so search does not fold every draft per keystroke.
    search_text: String,
}

impl Row {
    fn new(draft: Draft) -> Self {
        Self {
            title: draft.title().to_owned().into(),
            preview: draft.preview(PREVIEW_CHARS).into(),
            date: time_format::short(draft.modified_at()).into(),
            search_text: draft.content().to_lowercase(),
            draft,
        }
    }
}

impl DraftList {
    pub fn new(scope: Scope, drafts: Vec<Draft>) -> Self {
        let mut list = Self {
            scope,
            rows: Vec::new(),
            query: SearchQuery::default(),
            matches: Vec::new(),
        };
        list.set_drafts(scope, drafts);
        list
    }

    /// Replaces the drafts shown, keeping the search query.
    pub fn set_drafts(&mut self, scope: Scope, drafts: Vec<Draft>) {
        self.scope = scope;
        self.rows = drafts.into_iter().map(Row::new).collect();
        self.rematch();
    }

    /// Applies one draft's new state: updates its row, inserts it where
    /// `sort` places it, or drops it once it no longer belongs to the scope.
    pub fn apply(&mut self, draft: Draft, sort: Sort) {
        self.rows.retain(|row| row.draft.id() != draft.id());
        if self.scope.contains(&draft) {
            let at = self
                .rows
                .partition_point(|row| sort.compare(&row.draft, &draft) == Ordering::Less);
            self.rows.insert(at, Row::new(draft));
        }
        self.rematch();
    }

    pub fn remove(&mut self, id: DraftId) {
        self.rows.retain(|row| row.draft.id() != id);
        self.rematch();
    }

    fn row_at(&self, ix: IndexPath) -> Option<&Row> {
        self.matches
            .get(ix.row)
            .and_then(|row_ix| self.rows.get(*row_ix))
    }

    pub fn draft_at(&self, ix: IndexPath) -> Option<&Draft> {
        self.row_at(ix).map(|row| &row.draft)
    }

    pub fn position_of(&self, id: DraftId) -> Option<IndexPath> {
        self.matches
            .iter()
            .position(|row_ix| {
                self.rows
                    .get(*row_ix)
                    .is_some_and(|row| row.draft.id() == id)
            })
            .map(IndexPath::new)
    }

    /// The number of drafts shown (matching the search).
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    fn rematch(&mut self) {
        if self.query.is_empty() {
            self.matches = (0..self.rows.len()).collect();
            return;
        }
        self.matches = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.query.matches_lowercase(&row.search_text))
            .map(|(ix, _)| ix)
            .collect();
    }

    fn render_row(row: &Row, cx: &App) -> impl IntoElement {
        let theme = cx.theme();
        let has_preview = !row.preview.is_empty();

        // Every row has the same height (the list measures one row). A draft
        // without body text lets its title use the second line instead.
        h_flex()
            .w_full()
            .h_12()
            .my_1()
            .gap_2()
            .items_start()
            .overflow_hidden()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        div()
                            .w_full()
                            .font_weight(FontWeight::MEDIUM)
                            .map(|this| {
                                if has_preview {
                                    this.truncate()
                                } else {
                                    this.line_clamp(2).text_ellipsis()
                                }
                            })
                            .map(|this| {
                                if row.title.is_empty() {
                                    this.text_color(theme.muted_foreground).child("Empty draft")
                                } else {
                                    this.child(row.title.clone())
                                }
                            }),
                    )
                    .when(has_preview, |this| {
                        this.child(
                            div()
                                .w_full()
                                .truncate()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(row.preview.clone()),
                        )
                    }),
            )
            .child(
                h_flex()
                    .flex_none()
                    .h_6()
                    .gap_1()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .when(row.draft.is_flagged(), |this| {
                        this.child(Icon::new(IconName::Flag).xsmall().text_color(theme.warning))
                    })
                    .child(row.date.clone()),
            )
    }
}

/// The list as its own view, so the drafts window can cache it: typing in
/// the editor then does not re-render the list, and hovering rows does not
/// re-render the editor.
pub struct DraftListView {
    state: Entity<ListState<DraftList>>,
}

impl DraftListView {
    pub fn new(state: Entity<ListState<DraftList>>) -> Self {
        Self { state }
    }
}

impl Render for DraftListView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        List::new(&self.state).search_placeholder("Search").p_1()
    }
}

impl ListDelegate for DraftList {
    type Item = ListItem;

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = SearchQuery::parse(query);
        self.rematch();
        cx.notify();
        Task::ready(())
    }

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.matches.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let row = self.row_at(ix)?;
        Some(
            ListItem::new(ElementId::Uuid(row.draft.id().as_uuid()))
                .rounded(cx.theme().radius)
                .child(Self::render_row(row, cx)),
        )
    }

    fn render_empty(
        &mut self,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        let (icon, title, detail) = if self.has_query() {
            (
                IconName::Search,
                "No matching drafts",
                "Try other words, or remove a filter.",
            )
        } else {
            match self.scope {
                Scope::Inbox => (
                    IconName::Inbox,
                    "Inbox is empty",
                    "New and captured drafts land here.",
                ),
                Scope::Flagged => (
                    IconName::Flag,
                    "No flagged drafts",
                    "Flag a draft to find it here.",
                ),
                Scope::Archive => (
                    IconName::Archive,
                    "Archive is empty",
                    "Archive drafts you are done with to keep them out of the inbox.",
                ),
                Scope::All => (
                    IconName::FileText,
                    "No drafts yet",
                    "Start typing to create one.",
                ),
                Scope::Trash => (
                    IconName::Trash,
                    "Trash is empty",
                    "Drafts in the trash are deleted after 30 days.",
                ),
            }
        };

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .p_6()
            .text_center()
            .child(
                Icon::new(icon)
                    .large()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(div().font_weight(FontWeight::MEDIUM).child(title))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(detail),
            )
    }

    // Selection is owned by the list state and reported to the drafts window
    // through `ListEvent`, so the delegate keeps no copy of it.
    fn set_selected_index(
        &mut self,
        _: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
    }
}
