use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Bounds, ElementId, EmptyView, Entity, EntityId, Hsla,
    InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels, Point, RenderOnce,
    ScrollDelta, StatefulInteractiveElement, Styled, Window, canvas, div, fill, point, size,
};

use super::{
    shape::{Artboard, Guide, Shape, ShapeKind, polygon},
    view::{Frame, Viewport, marks, step},
};
use crate::{
    layout::seeded::use_seeded,
    theme::{ActiveTheme, Palette, TextSize},
};

pub(crate) type OnViewport = Rc<dyn Fn(Viewport, &mut Window, &mut App)>;

/// A drag that pans a canvas, and whose canvas it is.
struct Pan {
    owner: EntityId,
}

/// The canvas's box, and the pointer and viewport at the press of a pan under way.
#[derive(Default)]
struct Plane {
    bounds: Bounds<Pixels>,
    grip: Option<(Point<Pixels>, Viewport)>,
}

/// The view pixels of a canvas frame.
pub(crate) fn in_view(view: &Viewport, frame: &Frame) -> Frame {
    let (x, y) = view.to_view((frame.x, frame.y));
    Frame::new(x, y, frame.w * view.zoom, frame.h * view.zoom)
}

fn at(origin: Point<Pixels>, (x, y): (f32, f32)) -> Point<Pixels> {
    origin + point(Pixels::from(x), Pixels::from(y))
}

fn quad(origin: Point<Pixels>, frame: &Frame) -> Bounds<Pixels> {
    Bounds::new(
        at(origin, (frame.x, frame.y)),
        size(Pixels::from(frame.w), Pixels::from(frame.h)),
    )
}

fn outline(
    points: &[(f32, f32)],
    origin: Point<Pixels>,
    width: Pixels,
    closed: bool,
) -> PathBuilder {
    let mut path = PathBuilder::stroke(width);
    if let Some(first) = points.first() {
        path.move_to(at(origin, *first));
        points[1..]
            .iter()
            .for_each(|next| path.line_to(at(origin, *next)));
        if closed {
            path.close();
        }
    }
    path
}

fn solid(points: &[(f32, f32)], origin: Point<Pixels>) -> PathBuilder {
    let mut path = PathBuilder::fill();
    if let Some(first) = points.first() {
        path.move_to(at(origin, *first));
        points[1..]
            .iter()
            .for_each(|next| path.line_to(at(origin, *next)));
        path.close();
    }
    path
}

fn finish(path: PathBuilder, color: Hsla, window: &mut Window) {
    match path.build() {
        Ok(path) => window.paint_path(path, color),
        Err(error) => log::error!("canvas: a path failed to build: {error:#}"),
    }
}

/// Paints a shape's outline and wash in view pixels.
pub(crate) fn paint_shape(
    shape: &Shape,
    view: &Viewport,
    origin: Point<Pixels>,
    palette: &Palette,
    stroke: Pixels,
    window: &mut Window,
) {
    let ink = palette.hue(shape.hue, format_args!("shape {}", shape.key));
    let wash = ink.alpha(0.14);
    let frame = in_view(view, &shape.frame);
    match &shape.kind {
        ShapeKind::Rect => {
            window.paint_quad(fill(quad(origin, &frame), wash));
            let corners = [
                (frame.x, frame.y),
                (frame.right(), frame.y),
                (frame.right(), frame.bottom()),
                (frame.x, frame.bottom()),
            ];
            finish(outline(&corners, origin, stroke, true), ink, window);
        }
        ShapeKind::Ellipse | ShapeKind::Polygon(_) => {
            let sides = match shape.kind {
                ShapeKind::Polygon(sides) => sides,
                _ => 64,
            };
            let corners = polygon(&frame, sides);
            finish(solid(&corners, origin), wash, window);
            finish(outline(&corners, origin, stroke, true), ink, window);
        }
        ShapeKind::Line | ShapeKind::Arrow => {
            let (from, to) = ((frame.x, frame.y), (frame.right(), frame.bottom()));
            finish(outline(&[from, to], origin, stroke, false), ink, window);
            if shape.kind == ShapeKind::Arrow {
                let (dx, dy) = (to.0 - from.0, to.1 - from.1);
                let length = (dx * dx + dy * dy).sqrt().max(1.0);
                let (ux, uy) = (dx / length, dy / length);
                let head = f32::from(stroke) * 5.0;
                let back = (to.0 - ux * head, to.1 - uy * head);
                let left = (back.0 - uy * head / 2.0, back.1 + ux * head / 2.0);
                let right = (back.0 + uy * head / 2.0, back.1 - ux * head / 2.0);
                finish(solid(&[to, left, right], origin), ink, window);
            }
        }
        ShapeKind::Path(points) => {
            let points: Vec<(f32, f32)> = points
                .iter()
                .map(|(x, y)| view.to_view((shape.frame.x + x, shape.frame.y + y)))
                .collect();
            finish(outline(&points, origin, stroke, false), ink, window);
        }
        ShapeKind::Text(_) | ShapeKind::Note(_) => {}
    }
}

/// An endless plane: a dot grid that thins as it zooms out, rulers along the top and left, guides, artboards with their names, and shapes. A drag on empty space pans it from where it was pressed, a wheel scrolls it, and Command or Control with the wheel zooms about the pointer. The owner keeps the viewport; a new one from the owner shows at once.
#[derive(IntoElement)]
pub struct InfiniteCanvas {
    id: ElementId,
    viewport: Viewport,
    shapes: Vec<Shape>,
    artboards: Vec<Artboard>,
    guides: Vec<Guide>,
    rulers: bool,
    layers: Vec<AnyElement>,
    on_viewport: Option<OnViewport>,
}

impl InfiniteCanvas {
    pub fn new(id: impl Into<ElementId>, viewport: Viewport) -> Self {
        Self {
            id: id.into(),
            viewport,
            shapes: Vec::new(),
            artboards: Vec::new(),
            guides: Vec::new(),
            rulers: false,
            layers: Vec::new(),
            on_viewport: None,
        }
    }

    pub fn shapes(mut self, shapes: impl IntoIterator<Item = Shape>) -> Self {
        self.shapes = shapes.into_iter().collect();
        self
    }

    pub fn artboards(mut self, artboards: impl IntoIterator<Item = Artboard>) -> Self {
        self.artboards = artboards.into_iter().collect();
        self
    }

    pub fn guides(mut self, guides: impl IntoIterator<Item = Guide>) -> Self {
        self.guides = guides.into_iter().collect();
        self
    }

    /// Rulers along the top and the left, in canvas units.
    pub fn rulers(mut self) -> Self {
        self.rulers = true;
        self
    }

    /// Lays an element over the plane, in its view's pixels, as a selection or handles are.
    pub fn layer(mut self, element: impl IntoElement) -> Self {
        self.layers.push(element.into_any_element());
        self
    }

    /// Gets the viewport after each pan and zoom.
    pub fn on_viewport(
        mut self,
        handler: impl Fn(Viewport, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_viewport = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for InfiniteCanvas {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let local = use_seeded((id.clone(), "view"), self.viewport, window, cx);
        let plane: Entity<Plane> =
            window.use_keyed_state((id.clone(), "plane"), cx, |_, _| Plane::default());
        let view = local.read(cx).value;
        let bounds = plane.read(cx).bounds;
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let set: OnViewport = {
            let (local, on_viewport) = (local.clone(), self.on_viewport.clone());
            Rc::new(move |next, window, cx| {
                local.update(cx, |local, cx| {
                    local.value = next;
                    cx.notify();
                });
                if let Some(on_viewport) = &on_viewport {
                    on_viewport(next, window, cx);
                }
            })
        };
        let theme = cx.theme();
        let (colors, sizes) = (theme.colors.clone(), theme.canvas());
        let rem = window.rem_size();
        let (dot, hairline, stroke) = (sizes.dot, sizes.hairline, sizes.stroke.to_pixels(rem));
        let (ruler, side) = (sizes.ruler.to_pixels(rem), sizes.side.to_pixels(rem));
        let grid = step(view.zoom, f32::from(sizes.grid.to_pixels(rem)));
        let (left, top) = view.to_canvas((0.0, 0.0));
        let (right, bottom) = view.to_canvas((w, h));
        let dots: Vec<(f32, f32)> = marks(left, right, grid)
            .into_iter()
            .flat_map(|x| {
                marks(top, bottom, grid)
                    .into_iter()
                    .map(move |y| view.to_view((x, y)))
            })
            .collect();
        let words = |text: String, color| {
            div()
                .absolute()
                .whitespace_nowrap()
                .text_size(theme.text_size(TextSize::Xs))
                .text_color(color)
                .child(text)
        };
        let names: Vec<AnyElement> = self
            .artboards
            .iter()
            .map(|board| {
                let frame = in_view(&view, &board.frame);
                words(board.name.to_string(), colors.fg_muted)
                    .left(Pixels::from(frame.x))
                    .bottom(Pixels::from(h - frame.y))
                    .pb_1()
                    .into_any_element()
            })
            .collect();
        let texts: Vec<AnyElement> = self
            .shapes
            .iter()
            .filter(|shape| !shape.hidden)
            .filter_map(|shape| {
                let frame = in_view(&view, &shape.frame);
                let size = theme.text_size(TextSize::Base).to_pixels(rem) * view.zoom;
                let placed = |text: &str| {
                    div()
                        .absolute()
                        .left(Pixels::from(frame.x))
                        .top(Pixels::from(frame.y))
                        .w(Pixels::from(frame.w))
                        .h(Pixels::from(frame.h))
                        .overflow_hidden()
                        .text_size(size)
                        .line_height(size * 1.3)
                        .child(text.to_string())
                };
                match &shape.kind {
                    ShapeKind::Text(text) => {
                        Some(placed(text).text_color(colors.fg).into_any_element())
                    }
                    ShapeKind::Note(text) => {
                        let paper = colors.hue(shape.hue, format_args!("note {}", shape.key));
                        Some(
                            placed(text)
                                .p(sizes.note_inset.to_pixels(rem) * view.zoom)
                                .rounded(sizes.note_corner.to_pixels(rem) * view.zoom)
                                .bg(paper.alpha(0.22))
                                .text_color(colors.fg)
                                .into_any_element(),
                        )
                    }
                    _ => None,
                }
            })
            .collect();
        let tick = step(view.zoom, f32::from(sizes.label.to_pixels(rem)));
        let rulers = self.rulers.then(|| {
            let across = marks(left, right, tick).into_iter().map(|x| {
                words(format!("{}", x.round()), colors.fg_subtle)
                    .left(Pixels::from(view.to_view((x, 0.0)).0))
                    .top_0()
                    .pl_1()
                    .into_any_element()
            });
            let down = marks(top, bottom, tick).into_iter().map(|y| {
                words(format!("{}", y.round()), colors.fg_subtle)
                    .left_0()
                    .top(Pixels::from(view.to_view((0.0, y)).1))
                    .pl_0p5()
                    .into_any_element()
            });
            let strip = |element: gpui::Div| {
                element
                    .absolute()
                    .bg(colors.surface)
                    .border_color(colors.border)
            };
            [
                strip(div().top_0().left_0().right_0().h(ruler).border_b_1())
                    .children(across.collect::<Vec<_>>())
                    .into_any_element(),
                strip(div().top_0().left_0().bottom_0().w(side).border_r_1())
                    .children(down.collect::<Vec<_>>())
                    .into_any_element(),
            ]
        });
        let (shapes, artboards, guides, palette) =
            (self.shapes, self.artboards, self.guides, colors.clone());
        let painted = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let origin = bounds.origin;
                for (x, y) in &dots {
                    window.paint_quad(fill(
                        Bounds::new(at(origin, (*x, *y)), size(dot, dot)),
                        palette.border_strong,
                    ));
                }
                for board in &artboards {
                    let frame = in_view(&view, &board.frame);
                    window.paint_quad(
                        fill(quad(origin, &frame), palette.surface)
                            .border_widths(hairline)
                            .border_color(palette.border),
                    );
                }
                for shape in shapes.iter().filter(|shape| !shape.hidden) {
                    paint_shape(shape, &view, origin, &palette, stroke, window);
                }
                let line = palette.hue(3, "guide");
                for guide in &guides {
                    let (from, to) = match guide {
                        Guide::Vertical(x) => {
                            let x = view.to_view((*x, 0.0)).0;
                            ((x, 0.0), (x, h))
                        }
                        Guide::Horizontal(y) => {
                            let y = view.to_view((0.0, *y)).1;
                            ((0.0, y), (w, y))
                        }
                    };
                    finish(
                        outline(&[from, to], origin, stroke / 2.0, false),
                        line,
                        window,
                    );
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let measured = {
            let plane = plane.clone();
            canvas(
                move |bounds, window, cx| {
                    if plane.read(cx).bounds != bounds {
                        plane.update(cx, |plane, cx| {
                            plane.bounds = bounds;
                            cx.notify();
                        });
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
        };
        let owner = plane.entity_id();
        let (wheel, pan, drop) = (set.clone(), set, plane.clone());
        let (scrolled, dragged, pressed) = (plane.clone(), plane.clone(), plane);
        let (live, held) = (local.clone(), local.clone());
        div()
            .id(id)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(colors.sunken)
            .cursor_grab()
            .on_scroll_wheel(move |event, window, cx| {
                let bounds = scrolled.read(cx).bounds;
                let delta = match event.delta {
                    ScrollDelta::Pixels(delta) => (f32::from(delta.x), f32::from(delta.y)),
                    ScrollDelta::Lines(lines) => {
                        let line = f32::from(window.line_height());
                        (lines.x * line, lines.y * line)
                    }
                };
                if delta == (0.0, 0.0) {
                    return;
                }
                let pointer = event.position - bounds.origin;
                let about = (f32::from(pointer.x), f32::from(pointer.y));
                let now = live.read(cx).value;
                let next = match event.modifiers.platform || event.modifiers.control {
                    true => now.zoomed((-delta.1 / 240.0).exp(), about),
                    false => now.panned(delta),
                };
                cx.stop_propagation();
                wheel(next, window, cx);
            })
            .on_mouse_down(gpui::MouseButton::Left, move |event, _, cx| {
                let now = held.read(cx).value;
                pressed.update(cx, |plane, _| plane.grip = Some((event.position, now)))
            })
            .on_drag(Pan { owner }, |_, _, _, cx| cx.new(|_| EmptyView))
            .on_drag_move::<Pan>(move |event, window, cx| {
                if event.drag(cx).owner != owner {
                    return;
                }
                let Some((press, start)) = dragged.read(cx).grip else {
                    return;
                };
                let way = event.event.position - press;
                pan(
                    start.panned((f32::from(way.x), f32::from(way.y))),
                    window,
                    cx,
                );
            })
            .on_drop(move |_: &Pan, _, cx| drop.update(cx, |plane, _| plane.grip = None))
            .child(painted)
            .children(names)
            .children(texts)
            .children(self.layers)
            .children(rulers.into_iter().flatten())
            .child(measured)
    }
}
