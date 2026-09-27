mod advanced;
mod appearance;
mod editor;
mod network;
mod preferences;
mod search;
mod section;
mod shortcuts;
mod storage;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use advanced::{DeveloperModeToggle, FeatureFlags, Flag, Stage};
pub use appearance::{
    AccentColorPicker, Appearance, DensitySelector, FontSizeControl, ThemeSelector,
};
pub use editor::{ThemeDraft, ThemeEditor};
pub use network::{Proxy, ProxySettings, manual};
pub use preferences::{
    Notices, NotificationSettings, Privacy, PrivacySettings, Startup, StartupSettings,
};
pub use search::{SettingEntry, SettingsSearch, matching};
pub use section::{SettingsLayout, SettingsRow, SettingsSection};
pub use shortcuts::{KeyboardShortcutsList, Shortcut};
pub use storage::{Changed, ImportExportSettings, ResetToDefault, StorageSettings, Store};
