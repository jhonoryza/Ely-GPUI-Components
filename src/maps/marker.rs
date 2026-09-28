use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};

use super::LatLon;
use crate::{
    buttons::{ButtonVariant, IconButton},
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme, Elevation, IconSize, Radius, TextSize},
    typography::Ellipsis,
};

pub(crate) type OnMarker = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;

/// A pin at a place, keyed so it keeps its focus as the map moves; a label beside it, and a chart hue.
#[derive(Clone)]
pub struct MapMarker {
    pub(crate) key: SharedString,
    pub(crate) at: LatLon,
    label: Option<SharedString>,
    hue: Option<usize>,
}

impl MapMarker {
    pub fn new(key: impl Into<SharedString>, at: LatLon) -> Self {
        Self {
            key: key.into(),
            at,
            label: None,
            hue: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The chart hue the pin wears; the accent without one.
    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = Some(hue);
        self
    }

    /// The pin with its tip at a view point; pressable when the map has a handler.
    pub(crate) fn place(
        self,
        owner: &ElementId,
        (x, y): (f32, f32),
        on_marker: Option<OnMarker>,
        cx: &App,
    ) -> AnyElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let ink = match self.hue {
            Some(hue) => colors.hue(hue, format_args!("marker {}", self.key)),
            None => colors.accent,
        };
        let pin = div()
            .id((owner.clone(), format!("marker-{}", self.key)))
            .flex_none()
            .rounded(theme.radius(Radius::Sm))
            .border_1()
            .border_color(gpui::transparent_black())
            .child(Icon::new(IconName::MapPin).size(IconSize::Xl).color(ink))
            .when_some(on_marker, |pin, on_marker| {
                let key = self.key.clone();
                pin.tab_index(0)
                    .focus_ring(cx)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| on_marker(&key, window, cx))
            });
        let point = || {
            div()
                .absolute()
                .left(Pixels::from(x))
                .top(Pixels::from(y))
                .size_0()
                .flex()
                .items_end()
        };
        div()
            .absolute()
            .top_0()
            .left_0()
            .child(point().justify_center().child(pin))
            .when_some(self.label, |marker, label| {
                marker.child(
                    point().child(
                        div()
                            .flex_none()
                            .ml_3()
                            .mb_3()
                            .px_1p5()
                            .rounded(theme.radius(Radius::Sm))
                            .bg(colors.overlay)
                            .border_1()
                            .border_color(colors.border)
                            .text_size(theme.text_size(TextSize::Xs))
                            .text_color(colors.fg)
                            .whitespace_nowrap()
                            .child(label),
                    ),
                )
            })
            .into_any_element()
    }
}

/// A card that points at a place, over the map; its close button hands the owner `on_close`.
pub struct MapPopup {
    pub(crate) at: LatLon,
    title: SharedString,
    content: Option<AnyElement>,
    on_close: Option<OnClose>,
}

impl MapPopup {
    pub fn new(at: LatLon, title: impl Into<SharedString>) -> Self {
        Self {
            at,
            title: title.into(),
            content: None,
            on_close: None,
        }
    }

    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    pub(crate) fn closer(&self) -> Option<OnClose> {
        self.on_close.clone()
    }

    /// The card at its point in a view of `size`; none when the point lies outside.
    pub(crate) fn place(
        self,
        owner: &ElementId,
        point: (f32, f32),
        size: (f32, f32),
        window: &Window,
        cx: &App,
    ) -> Option<AnyElement> {
        let theme = cx.theme();
        let colors = &theme.colors;
        let rem = window.rem_size();
        let sizes = theme.maps();
        let margin = f32::from(sizes.margin.to_pixels(rem));
        let lift = f32::from(theme.icon_size(IconSize::Xl).to_pixels(rem)) + margin;
        let spot = spot(
            point,
            size,
            f32::from(sizes.popup.to_pixels(rem)),
            margin,
            lift,
        )?;
        let card = div()
            .id((owner.clone(), "popup"))
            .absolute()
            .left(Pixels::from(spot.left))
            .w(Pixels::from(spot.width))
            .occlude()
            .flex()
            .items_start()
            .gap_2()
            .p_3()
            .rounded(theme.radius(Radius::Lg))
            .bg(colors.overlay)
            .border_1()
            .border_color(colors.border)
            .shadow(theme.elevation(Elevation::Floating))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Sm))
                            .text_color(colors.fg)
                            .child(Ellipsis::new(self.title)),
                    )
                    .children(self.content),
            )
            .when_some(self.on_close, |card, on_close| {
                card.child(
                    IconButton::new((owner.clone(), "popup-close"), IconName::X)
                        .variant(ButtonVariant::Ghost)
                        .tooltip("Close")
                        .on_click(move |_, window, cx| on_close(window, cx)),
                )
            });
        Some(
            match spot.edge {
                Edge::Bottom(bottom) => card.bottom(Pixels::from(bottom)),
                Edge::Top(top) => card.top(Pixels::from(top)),
            }
            .into_any_element(),
        )
    }
}

/// Which edge of a popup holds it: its bottom above the point, or its top below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Edge {
    Bottom(f32),
    Top(f32),
}

/// Where a popup lies in a view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Spot {
    pub left: f32,
    pub width: f32,
    pub edge: Edge,
}

/// A popup `width` wide at most, centered on its point and held `margin` inside the view, opening toward the view's larger half: above clears a pin by `lift`. None when the point lies outside.
pub(crate) fn spot(
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    width: f32,
    margin: f32,
    lift: f32,
) -> Option<Spot> {
    if !(0.0..=w).contains(&x) || !(0.0..=h).contains(&y) {
        return None;
    }
    let width = width.min(w - margin * 2.0).max(0.0);
    let left = (x - width / 2.0).clamp(margin, (w - margin - width).max(margin));
    let edge = if y > h / 2.0 {
        Edge::Bottom(h - y + lift)
    } else {
        Edge::Top(y + margin)
    };
    Some(Spot { left, width, edge })
}

#[cfg(test)]
mod tests {
    use super::{Edge, Spot, spot};

    const VIEW: (f32, f32) = (400.0, 300.0);

    #[test]
    fn a_popup_centers_on_its_point_and_stays_inside() {
        let at = |x: f32, y: f32| spot((x, y), VIEW, 240.0, 8.0, 32.0);
        assert_eq!(
            at(200.0, 250.0),
            Some(Spot {
                left: 80.0,
                width: 240.0,
                edge: Edge::Bottom(82.0)
            })
        );
        assert_eq!(
            at(10.0, 250.0).map(|spot| spot.left),
            Some(8.0),
            "held off the left edge"
        );
        assert_eq!(
            at(395.0, 250.0).map(|spot| spot.left),
            Some(152.0),
            "held off the right edge"
        );
        assert_eq!(
            at(200.0, 40.0).map(|spot| spot.edge),
            Some(Edge::Top(48.0)),
            "a point up top opens below"
        );
        assert_eq!(at(-1.0, 100.0), None, "a point outside draws none");
        assert_eq!(at(200.0, 301.0), None);
    }

    #[test]
    fn a_narrow_map_narrows_its_popup() {
        let narrow = spot((100.0, 100.0), (200.0, 300.0), 240.0, 8.0, 32.0).expect("inside");
        assert_eq!((narrow.left, narrow.width), (8.0, 184.0));
    }
}
