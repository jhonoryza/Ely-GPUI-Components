use std::{cell::RefCell, collections::HashSet, f64::consts::LOG2_E, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, Context, Entity, FocusHandle, ImageSource, IntoElement, KeyUpEvent, Keystroke,
    Modifiers, MouseButton, ParentElement, Render, RenderImage, ScrollDelta, ScrollWheelEvent,
    SharedString, Styled, TestAppContext, TouchPhase, VisualTestContext, Window, div, point, px,
};

use super::{LatLon, LocationPicker, MapMarker, MapPopup, MapView, MapViewport, Tile};
use crate::{
    primitives::{FocusNext, FocusScope},
    theme::Theme,
};

/// The view inside the map's 1px border, in pixels.
const SIZE: (f32, f32) = (398.0, 298.0);
const TILE: f32 = 256.0;

fn start() -> MapViewport {
    MapViewport::new(LatLon::new(0.0, 0.0), 2.0)
}

/// A view that shows one map part and keeps what it heard.
struct Stage {
    root: FocusHandle,
    part: fn(&Stage, Entity<Stage>) -> AnyElement,
    views: Vec<MapViewport>,
    said: Vec<SharedString>,
    tiles: Rc<RefCell<HashSet<Tile>>>,
}

impl Render for Stage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root).root().size_full().child(
            div()
                .w(px(400.0))
                .h(px(300.0))
                .child((self.part)(self, cx.entity())),
        )
    }
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

fn stage(
    part: fn(&Stage, Entity<Stage>) -> AnyElement,
    cx: &mut TestAppContext,
) -> (Entity<Stage>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
        cx.bind_keys([gpui::KeyBinding::new("tab", FocusNext, None)]);
    });
    let (host, cx) = cx.add_window_view(|_, cx| Stage {
        root: cx.focus_handle(),
        part,
        views: Vec::new(),
        said: Vec::new(),
        tiles: Rc::default(),
    });
    settle(cx);
    (host, cx)
}

fn hear(owner: &Entity<Stage>, view: MapViewport, cx: &mut gpui::App) {
    owner.update(cx, |stage, cx| {
        stage.views.push(view);
        cx.notify();
    });
}

fn say(owner: &Entity<Stage>, words: &str, cx: &mut gpui::App) {
    owner.update(cx, |stage, cx| {
        stage.said.push(words.to_string().into());
        cx.notify();
    });
}

fn last(host: &Entity<Stage>, cx: &mut VisualTestContext) -> MapViewport {
    *host
        .read_with(cx, |stage, _| stage.views.clone())
        .last()
        .expect("the map moved")
}

fn said(host: &Entity<Stage>, cx: &mut VisualTestContext) -> Vec<SharedString> {
    host.read_with(cx, |stage, _| stage.said.clone())
}

fn picture() -> ImageSource {
    let frame = image::Frame::new(image::RgbaImage::new(1, 1));
    ImageSource::Render(Arc::new(RenderImage::new(vec![frame])))
}

fn map(stage: &Stage, owner: Entity<Stage>) -> AnyElement {
    let tiles = stage.tiles.clone();
    let heard = owner.clone();
    MapView::new("map", start())
        .tiles(move |tile| {
            tiles.borrow_mut().insert(tile);
            picture()
        })
        .markers([MapMarker::new("middle", LatLon::new(0.0, 0.0)).label("Null Island")])
        .on_marker(move |key, _, cx| say(&heard, key, cx))
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

fn at(x: f32, y: f32) -> gpui::Point<gpui::Pixels> {
    point(px(x), px(y))
}

/// A view point as the window has it, past the map's border.
fn window_at((x, y): (f32, f32)) -> gpui::Point<gpui::Pixels> {
    at(x + 1.0, y + 1.0)
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01
}

/// Focuses the map with a press on open water, away from its pin and buttons.
fn focus_map(cx: &mut VisualTestContext) {
    cx.simulate_click(window_at((40.0, 250.0)), Modifiers::none());
    settle(cx);
}

#[gpui::test]
fn a_drag_pans_the_map_with_the_pointer(cx: &mut TestAppContext) {
    let (host, cx) = stage(map, cx);
    let place = start().to_geo((60.0, 200.0), SIZE, TILE);
    let none = Modifiers::none();
    cx.simulate_mouse_move(window_at((60.0, 200.0)), None, none);
    cx.simulate_mouse_down(window_at((60.0, 200.0)), MouseButton::Left, none);
    for (x, y) in [(70.0, 190.0), (90.0, 175.0), (110.0, 170.0)] {
        cx.simulate_mouse_move(window_at((x, y)), Some(MouseButton::Left), none);
        settle(cx);
    }
    cx.simulate_mouse_up(window_at((110.0, 170.0)), MouseButton::Left, none);
    settle(cx);
    let moved = last(&host, cx).to_view(place, SIZE, TILE);
    assert!(
        near(moved, (110.0, 170.0)),
        "the place followed the pointer: {moved:?}"
    );
}

#[gpui::test]
fn command_and_the_wheel_zoom_about_the_pointer(cx: &mut TestAppContext) {
    let (host, cx) = stage(map, cx);
    let place = start().to_geo((300.0, 60.0), SIZE, TILE);
    cx.simulate_event(ScrollWheelEvent {
        position: window_at((300.0, 60.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-240.0))),
        modifiers: Modifiers::command(),
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
    let view = last(&host, cx);
    assert!(
        (view.zoom - (2.0 + LOG2_E)).abs() < 1e-6,
        "a turn of 240 zooms by e: {}",
        view.zoom
    );
    assert!(near(view.to_view(place, SIZE, TILE), (300.0, 60.0)));
    cx.simulate_event(ScrollWheelEvent {
        position: window_at((300.0, 60.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-50.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
    let panned = last(&host, cx).to_view(place, SIZE, TILE);
    assert!(
        near(panned, (300.0, 10.0)),
        "a plain wheel pans: {panned:?}"
    );
}

#[gpui::test]
fn keys_pan_and_zoom_the_focused_map(cx: &mut TestAppContext) {
    let (host, cx) = stage(map, cx);
    focus_map(cx);
    let middle = (SIZE.0 / 2.0, SIZE.1 / 2.0);
    let place = start().to_geo(middle, SIZE, TILE);
    press("left", cx);
    let step = SIZE.0.min(SIZE.1) / 4.0;
    let moved = last(&host, cx).to_view(place, SIZE, TILE);
    assert!(
        near(moved, (middle.0 + step, middle.1)),
        "left shows what lies west: {moved:?}"
    );
    press("=", cx);
    assert_eq!(last(&host, cx).zoom, 3.0);
    press("-", cx);
    assert_eq!(last(&host, cx).zoom, 2.0);
    let count = host.read_with(cx, |stage, _| stage.views.len());
    press("cmd-left", cx);
    assert_eq!(
        host.read_with(cx, |stage, _| stage.views.len()),
        count,
        "the app keeps Command"
    );
}

#[gpui::test]
fn a_marker_takes_a_press_and_enter_by_its_key(cx: &mut TestAppContext) {
    let (host, cx) = stage(map, cx);
    let (x, y) = start().to_view(LatLon::new(0.0, 0.0), SIZE, TILE);
    cx.simulate_click(window_at((x, y - 12.0)), Modifiers::none());
    settle(cx);
    assert_eq!(said(&host, cx), ["middle"], "a press on the pin");
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["middle", "middle"],
        "Tab reaches the pin, Enter presses it"
    );
}

fn quiet(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    MapView::new("map", start())
        .markers([MapMarker::new("middle", LatLon::new(0.0, 0.0))])
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

/// A map without a marker handler leaves its pins out of Tab: the next stop is the zoom button.
#[gpui::test]
fn pins_without_a_handler_take_no_tab(cx: &mut TestAppContext) {
    let (host, cx) = stage(quiet, cx);
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(last(&host, cx).zoom, 3.0, "Enter on the zoom button");
    press("tab", cx);
    press("enter", cx);
    assert_eq!(last(&host, cx).zoom, 2.0, "the next stop zooms out");
}

fn tiled(stage: &Stage, _: Entity<Stage>) -> AnyElement {
    let tiles = stage.tiles.clone();
    MapView::new("map", MapViewport::new(LatLon::new(0.0, 0.0), 1.0))
        .tiles(move |tile| {
            tiles.borrow_mut().insert(tile);
            picture()
        })
        .into_any_element()
}

fn requested(host: &Entity<Stage>, cx: &mut VisualTestContext) -> HashSet<Tile> {
    host.read_with(cx, |stage, _| stage.tiles.borrow().clone())
}

#[gpui::test]
fn the_host_supplies_each_tile_the_view_covers(cx: &mut TestAppContext) {
    let (host, cx) = stage(tiled, cx);
    let expected: HashSet<Tile> = [(0, 0), (1, 0), (0, 1), (1, 1)]
        .map(|(x, y)| Tile { z: 1, x, y })
        .into();
    assert_eq!(requested(&host, cx), expected);
}

fn capped(stage: &Stage, _: Entity<Stage>) -> AnyElement {
    let tiles = stage.tiles.clone();
    MapView::new("map", MapViewport::new(LatLon::new(0.0, 0.0), 1.0))
        .tile_zooms(0, 0)
        .tiles(move |tile| {
            tiles.borrow_mut().insert(tile);
            picture()
        })
        .into_any_element()
}

#[gpui::test]
fn past_the_hosts_zooms_one_tile_scales(cx: &mut TestAppContext) {
    let (host, cx) = stage(capped, cx);
    assert_eq!(
        requested(&host, cx),
        HashSet::from([Tile { z: 0, x: 0, y: 0 }])
    );
}

fn with_popup(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    let closed = owner.clone();
    MapView::new("map", start())
        .popup(
            MapPopup::new(LatLon::new(0.0, 0.0), "Null Island")
                .child("Where the equator meets the prime meridian.")
                .on_close(move |_, cx| say(&closed, "closed", cx)),
        )
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

#[gpui::test]
fn the_popup_closes_by_its_button_and_by_escape(cx: &mut TestAppContext) {
    let (host, cx) = stage(with_popup, cx);
    focus_map(cx);
    press("escape", cx);
    assert_eq!(said(&host, cx), ["closed"], "Escape on the map");
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["closed", "closed"],
        "Tab reaches the close button first"
    );
}

fn far_popup(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    MapView::new("map", start())
        .popup(MapPopup::new(LatLon::new(-60.0, 120.0), "Far").on_close(|_, _| {}))
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

/// A popup whose place lies outside the view is not drawn, so Tab passes to the zoom button.
#[gpui::test]
fn a_popup_away_from_the_view_hides(cx: &mut TestAppContext) {
    let (host, cx) = stage(far_popup, cx);
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(last(&host, cx).zoom, 3.0);
}

fn picker(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    LocationPicker::new("picker", MapView::new("pick-map", start()))
        .on_pick(move |at, _, cx| say(&owner, &format!("{:.4},{:.4}", at.lat, at.lon), cx))
        .into_any_element()
}

#[gpui::test]
fn the_picker_picks_the_center_as_the_map_moves(cx: &mut TestAppContext) {
    let (host, cx) = stage(picker, cx);
    cx.simulate_click(at(40.0, 40.0), Modifiers::none());
    settle(cx);
    press("right", cx);
    press("down", cx);
    let heard = said(&host, cx);
    assert_eq!(heard.len(), 2, "one pick per move: {heard:?}");
    let first: Vec<f64> = heard[0]
        .split(',')
        .map(|part| part.parse().expect("a number"))
        .collect();
    assert_eq!(first[0], 0.0, "right moves east along the equator");
    assert!(first[1] > 0.0);
    let second: Vec<f64> = heard[1]
        .split(',')
        .map(|part| part.parse().expect("a number"))
        .collect();
    assert!(
        second[0] < 0.0 && second[1] == first[1],
        "down moves south: {heard:?}"
    );
}

fn loud(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    LocationPicker::new(
        "picker",
        MapView::new("pick-map", start()).on_viewport(move |view, _, cx| hear(&owner, view, cx)),
    )
    .into_any_element()
}

#[gpui::test]
#[should_panic(expected = "it hears the map's moves")]
fn a_picker_whose_map_hears_its_own_moves_fails(cx: &mut TestAppContext) {
    stage(loud, cx);
}

fn nearest(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    MapView::new(
        "map",
        MapViewport::new(LatLon::new(0.0, 0.0), MapViewport::ZOOMS.1),
    )
    .on_viewport(move |view, _, cx| hear(&owner, view, cx))
    .into_any_element()
}

/// At the nearest zoom the zoom-in button rests and leaves Tab, so the next stop zooms out.
#[gpui::test]
fn the_zoom_buttons_rest_at_the_limits(cx: &mut TestAppContext) {
    let (host, cx) = stage(nearest, cx);
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(last(&host, cx).zoom, MapViewport::ZOOMS.1 - 1.0);
}

#[test]
#[should_panic(expected = "tile zooms 4 to 2")]
fn tile_zooms_out_of_order_fail() {
    let _ = MapView::new("map", start()).tile_zooms(4, 2);
}
