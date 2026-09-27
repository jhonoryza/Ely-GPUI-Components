use std::rc::Rc;

use gpui::{App, Window};

use crate::lists::DirEntry;

mod columns;
mod explorer;
mod grid;
mod icon;
mod item;

type OnEntry = Rc<dyn Fn(&DirEntry, &mut Window, &mut App)>;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use explorer::{FileExplorer, FileView};
pub use grid::FileGrid;
pub use icon::FileIcon;
pub use item::FileItem;
