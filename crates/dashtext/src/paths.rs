use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use directories::ProjectDirs;

/// Filesystem locations used by Dashtext.
///
/// On Linux these follow the XDG Base Directory specification:
///
/// | What                      | Where                                   |
/// | ------------------------- | --------------------------------------- |
/// | Draft library (user data) | `$XDG_DATA_HOME/dashtext/library.db`    |
/// | Instance socket           | `$XDG_RUNTIME_DIR/dashtext/dashtext.sock` |
///
/// macOS and Windows use their platform conventions via `directories`.
#[derive(Clone, Debug)]
pub struct Paths {
    data_dir: PathBuf,
    runtime_dir: PathBuf,
}

impl Paths {
    pub fn resolve() -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from("app", "dashtext", "Dashtext")
            .context("could not determine the home directory")?;
        let data_dir = dirs.data_dir().to_path_buf();
        // Without `$XDG_RUNTIME_DIR` (or off Linux) keep the socket beside
        // the library; it is private to the user either way.
        let runtime_dir = dirs
            .runtime_dir()
            .map_or_else(|| data_dir.clone(), Path::to_path_buf);
        Ok(Self {
            data_dir,
            runtime_dir,
        })
    }

    /// Every location inside `dir`, for tests.
    #[cfg(test)]
    pub fn in_dir(dir: &Path) -> Self {
        Self {
            data_dir: dir.to_path_buf(),
            runtime_dir: dir.to_path_buf(),
        }
    }

    /// Creates the directories that must exist before the app writes to them.
    pub fn ensure(&self) -> anyhow::Result<()> {
        for dir in [&self.data_dir, &self.runtime_dir] {
            create_private_dir(dir)
                .with_context(|| format!("could not create {}", dir.display()))?;
        }
        Ok(())
    }

    pub fn library(&self) -> PathBuf {
        self.data_dir.join("library.db")
    }

    pub fn socket(&self) -> PathBuf {
        self.runtime_dir.join("dashtext.sock")
    }
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt as _;

    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}
