mod factor;
mod keys;
mod login;
mod mail;
mod oauth;
mod profile;
mod sessions;
mod signup;
mod switcher;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use factor::TwoFactorInput;
pub use keys::{ApiKey, ApiKeyManager};
pub use login::{Login, LoginForm};
pub use mail::{ForgotPassword, MagicLinkForm};
pub use oauth::OAuthButtons;
pub use profile::{Profile, ProfileCard, ProfileEditor};
pub use sessions::{Session, SessionList};
pub use signup::{Signup, SignupForm, password_rules};
pub use switcher::{Account, AccountSwitcher, UserMenu, Workspace, WorkspaceSwitcher};
