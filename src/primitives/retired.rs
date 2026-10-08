//! Images a view stopped drawing, freed from the window's atlas.

use std::sync::Arc;

use gpui::{RenderImage, Window};

/// Images to free; GPUI keeps each painted image until told.
#[derive(Default)]
pub struct Retired(Vec<Arc<RenderImage>>);

impl Retired {
    /// An image no longer drawn; freed at the next paint.
    pub fn push(&mut self, image: Arc<RenderImage>) {
        self.0.push(image);
    }

    /// Frees every retired image; call while painting.
    pub fn release(&mut self, window: &mut Window) {
        for image in self.0.drain(..) {
            if let Err(error) = window.drop_image(image) {
                log::error!("images: a retired image stayed: {error}");
            }
        }
    }

    /// Whether nothing waits to be freed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
