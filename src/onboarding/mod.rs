mod checklist;
mod highlight;
mod hotspot;
mod wizard;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use checklist::{SetupChecklist, SetupTask};
pub use highlight::FeatureHighlight;
pub use hotspot::Hotspot;
pub use wizard::{OnboardingStep, OnboardingWizard};
