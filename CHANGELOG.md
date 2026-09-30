# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project attempts to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
## [${version}]
### Added - for new features
### Changed - for changes in existing functionality
### Deprecated - for soon-to-be removed features
### Removed - for now removed features
### Fixed - for any bug fixes
### Security - in case of vulnerabilities
[${version}]: https://github.com/joshuadavidthomas/dashtext/releases/tag/v${version}
-->

## [Unreleased]

### Changed

- Rewrote Dashtext as a native Rust application using GPUI and GPUI Component, replacing the Tauri and SvelteKit app. Drafts from 0.3 are not migrated.
- Drafts are stored in a SQLite library under the XDG data directory (`~/.local/share/dashtext/library.db`).

### Added

- Quick capture window that keeps unsaved text across closes and restarts
- Inbox, Flagged, Archive, All and Trash views; the trash is emptied after 30 days
- Search with required words, `"exact phrases"` and `-exclusions`
- `dashtext capture` and `dashtext new` commands, handled by the running instance when there is one
- Application menus and keyboard shortcuts for every draft command
- Desktop entry with a Quick Capture action

### Removed

- Vim mode, the web version and the built-in updater

## [0.3.1]

### Fixed

- Fixed flash of unstyled content (FOUC) on app startup

## [0.3.0]

### Added

- Quick capture window for rapid note-taking

## [0.2.0]

### Added

- Added auto-updating support for the application

## [0.1.0]

### Added

- Text editor powered by CodeMirror 6
- Draft persistence with local SQLite database

### New Contributors

- Josh Thomas <josh@joshthomas.dev> (maintainer)

[unreleased]: https://github.com/joshuadavidthomas/dashtext/compare/v0.3.1...HEAD
[0.1.0]: https://github.com/joshuadavidthomas/dashtext/releases/tag/v0.1.0
[0.2.0]: https://github.com/joshuadavidthomas/dashtext/releases/tag/v0.2.0
[0.3.0]: https://github.com/joshuadavidthomas/dashtext/releases/tag/v0.3.0
[0.3.1]: https://github.com/joshuadavidthomas/dashtext/releases/tag/v0.3.1
