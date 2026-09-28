use gpui::{AnyElement, Entity, IntoElement, Modifiers, TestAppContext, VisualTestContext};

use super::{Stage, at, focus_map, hear, last, moves, press, said, say, settle, stage, start};
use crate::maps::{LatLon, LocationPicker, MapMarker, MapPopup, MapView};

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

/// A shown popup holds focus: Escape closes it, and Tab reaches its close button.
#[gpui::test]
fn the_popup_closes_by_escape_and_by_its_button(cx: &mut TestAppContext) {
    let (host, cx) = stage(with_popup, cx);
    press("escape", cx);
    assert_eq!(said(&host, cx), ["closed"], "Escape in the card it focused");
    press("tab", cx);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["closed", "closed"],
        "Tab goes to the close button"
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

fn opening(stage: &Stage, owner: Entity<Stage>) -> AnyElement {
    let (opened, closed) = (owner.clone(), owner.clone());
    let map = MapView::new("map", start())
        .markers([MapMarker::new("middle", LatLon::new(0.0, 0.0))])
        .on_marker(move |_, _, cx| {
            opened.update(cx, |stage, cx| {
                stage.open = true;
                stage.said.push("opened".into());
                cx.notify();
            })
        })
        .on_viewport(move |view, _, cx| hear(&owner, view, cx));
    match stage.open {
        true => map
            .popup(
                MapPopup::new(LatLon::new(0.0, 0.0), "Null Island").on_close(move |_, cx| {
                    closed.update(cx, |stage, cx| {
                        stage.open = false;
                        stage.said.push("closed".into());
                        cx.notify();
                    })
                }),
            )
            .into_any_element(),
        false => map.into_any_element(),
    }
}

/// Opens the popup from its pin by keys.
fn open_by_pin(cx: &mut VisualTestContext) {
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
}

/// The close button takes the card away and focus returns to the pin that opened it: Enter opens it again, and the arrows still pan.
#[gpui::test]
fn closing_the_popup_by_its_button_hands_focus_back(cx: &mut TestAppContext) {
    let (host, cx) = stage(opening, cx);
    open_by_pin(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(said(&host, cx), ["opened", "closed"]);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["opened", "closed", "opened"],
        "the pin holds focus"
    );
    let before = moves(&host, cx);
    press("left", cx);
    assert_eq!(moves(&host, cx), before + 1);
}

/// Escape takes the card away and focus returns to the pin.
#[gpui::test]
fn escape_from_the_popup_hands_focus_back(cx: &mut TestAppContext) {
    let (host, cx) = stage(opening, cx);
    open_by_pin(cx);
    press("escape", cx);
    assert_eq!(said(&host, cx), ["opened", "closed"]);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["opened", "closed", "opened"],
        "the pin holds focus"
    );
}
