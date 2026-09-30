use std::borrow::Cow;

use gpui_kit::AssetSource;
use gpui_kit::Result;
use gpui_kit::SharedString;
use gpui_kit::assets::Assets;
use gpui_kit::assets::icon_assets;

// Lucide icons the app uses beyond the component library's default set.
icon_assets!(
    ExtraIcons,
    [
        Archive,
        ArchiveRestore,
        Flag,
        FlagOff,
        SquarePen,
        Trash,
        Zap
    ]
);

/// The application's asset source: the extra icons above, then the component
/// library's defaults.
#[derive(Clone, Copy, Debug, Default)]
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match ExtraIcons.load(path)? {
            Some(data) => Ok(Some(data)),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = ExtraIcons.list(path)?;
        paths.extend(Assets.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
