use gpui::{App, IntoElement, RenderOnce, Window};

use crate::{
    lists::DirEntry,
    primitives::{Icon, IconName, file_icon},
    theme::{ActiveTheme, IconSize},
};

/// A file's icon by its name's extension, or a folder's, open or shut.
#[derive(IntoElement)]
pub struct FileIcon {
    icon: IconName,
    size: IconSize,
}

impl FileIcon {
    pub fn file(name: impl AsRef<str>) -> Self {
        Self {
            icon: file_icon(name.as_ref()),
            size: IconSize::Md,
        }
    }

    pub fn folder(open: bool) -> Self {
        Self {
            icon: if open {
                IconName::FolderOpen
            } else {
                IconName::Folder
            },
            size: IconSize::Md,
        }
    }

    /// The icon for a listing's entry: a shut folder, or the file's by its name.
    pub fn entry(entry: &DirEntry) -> Self {
        match entry.is_folder() {
            true => Self::folder(false),
            false => Self::file(entry.name()),
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for FileIcon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        Icon::new(self.icon)
            .size(self.size)
            .color(cx.theme().colors.fg_muted)
    }
}
