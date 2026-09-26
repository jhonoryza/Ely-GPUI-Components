use gpui::{
    Context, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, point, px,
};

use super::{press, settle, setup, tab};
use crate::generative::{InpaintCanvas, MaskBrush, MaskStroke};

/// A canvas 400 wide over a picture twice as wide as tall, its brush and brush panel, and the strokes and brushes heard.
struct Painting {
    strokes: Vec<MaskStroke>,
    brush: (f32, bool),
    undone: usize,
}

impl Render for Painting {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (stroked, keyed, set, undo) = (cx.entity(), cx.entity(), cx.entity(), cx.entity());
        let (radius, erase) = self.brush;
        div()
            .w(px(400.0))
            .child(
                InpaintCanvas::new("canvas", "missing.jpg", 2.0, self.strokes.clone())
                    .brush(radius, erase)
                    .on_stroke(move |stroke, _, cx| {
                        stroked.update(cx, |view, cx| {
                            view.strokes.push(stroke);
                            cx.notify();
                        })
                    })
                    .on_brush(move |radius, erase, _, cx| {
                        keyed.update(cx, |view, cx| {
                            view.brush = (radius, erase);
                            cx.notify();
                        })
                    }),
            )
            .child(
                MaskBrush::new("brush", radius, erase, self.strokes.len())
                    .on_brush(move |radius, erase, _, cx| {
                        set.update(cx, |view, cx| {
                            view.brush = (radius, erase);
                            cx.notify();
                        })
                    })
                    .on_undo(move |_, cx| {
                        undo.update(cx, |view, cx| {
                            view.strokes.pop();
                            view.undone += 1;
                            cx.notify();
                        })
                    }),
            )
    }
}

fn painting(cx: &mut TestAppContext) -> (Entity<Painting>, &mut VisualTestContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Painting {
        strokes: Vec::new(),
        brush: (0.04, false),
        undone: 0,
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (view, cx)
}

#[gpui::test]
fn a_drag_lands_a_stroke_in_shares_of_the_picture(cx: &mut TestAppContext) {
    let (view, cx) = painting(cx);
    let none = Modifiers::none();
    cx.simulate_mouse_down(point(px(100.0), px(50.0)), MouseButton::Left, none);
    for x in [150.0, 200.0, 250.0] {
        cx.simulate_mouse_move(point(px(x), px(50.0)), MouseButton::Left, none);
    }
    cx.simulate_mouse_up(point(px(300.0), px(50.0)), MouseButton::Left, none);
    settle(cx);
    let strokes = view.read_with(cx, |view, _| view.strokes.clone());
    assert_eq!(strokes.len(), 1, "{strokes:?}");
    let (first, last) = (
        strokes[0].points[0],
        *strokes[0].points.last().expect("points"),
    );
    let near = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
    assert!(
        near(first, (0.25, 0.25)) && near(last, (0.75, 0.25)),
        "{first:?} to {last:?}"
    );
    assert_eq!((strokes[0].radius, strokes[0].erase), (0.04, false));
}

#[gpui::test]
fn keys_on_the_canvas_size_the_brush_and_swap_to_erasing(cx: &mut TestAppContext) {
    let (view, cx) = painting(cx);
    tab(1, cx);
    press("]", cx);
    press("x", cx);
    let brush = view.read_with(cx, |view, _| view.brush);
    assert!((brush.0 - 0.05).abs() < 1e-6 && brush.1, "{brush:?}");
}

#[gpui::test]
fn undo_waits_for_a_stroke(cx: &mut TestAppContext) {
    let (view, cx) = painting(cx);
    tab(4, cx);
    press("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.undone),
        0,
        "nothing to undo yet"
    );
    view.update(cx, |view, cx| {
        view.strokes.push(MaskStroke {
            points: vec![(0.5, 0.5)],
            radius: 0.04,
            erase: false,
        });
        cx.notify();
    });
    settle(cx);
    cx.update(|window, _| window.blur());
    tab(4, cx);
    press("enter", cx);
    assert_eq!(
        view.read_with(cx, |view, _| (view.undone, view.strokes.len())),
        (1, 0)
    );
}
