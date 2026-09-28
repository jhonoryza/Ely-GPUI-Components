use gpui::{AnyElement, Entity, IntoElement, TestAppContext};

use super::{SIZE, Stage, TILE, focus_map, hear, last, moves, press, said, say, stage};
use crate::maps::{
    GeoHeatmap, GeoJsonLayer, LatLon, MapMarker, MapView, MapViewport, RouteLine, read_geojson,
};

const CROWD: [(&str, f64, f64); 3] = [
    ("london", 51.5, -0.1),
    ("paris", 48.9, 2.35),
    ("brussels", 50.85, 4.35),
];

fn europe() -> MapViewport {
    MapViewport::new(LatLon::new(45.0, 0.0), 2.0)
}

fn crowd(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    let heard = owner.clone();
    let near = CROWD.map(|(key, lat, lon)| MapMarker::new(key, LatLon::new(lat, lon)));
    MapView::new("map", europe())
        .markers(
            near.into_iter()
                .chain([MapMarker::new("tokyo", LatLon::new(35.7, 139.7))]),
        )
        .cluster_markers()
        .on_marker(move |key, _, cx| say(&heard, key, cx))
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

/// Three capitals in one square of the world gather; the cluster's press shows them whole, apart, and each pin then takes its own press.
#[gpui::test]
fn crowded_pins_gather_and_a_press_shows_them_whole(cx: &mut TestAppContext) {
    let (host, cx) = stage(crowd, cx);
    focus_map(cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        Vec::<gpui::SharedString>::new(),
        "the first stop is the cluster"
    );
    let fit = last(&host, cx);
    assert!(fit.zoom > europe().zoom, "nearer in: {}", fit.zoom);
    for (key, lat, lon) in CROWD {
        let (x, y) = fit.to_view(LatLon::new(lat, lon), SIZE, TILE);
        assert!(
            (0.0..=SIZE.0).contains(&x) && (0.0..=SIZE.1).contains(&y),
            "{key} in view at {x}, {y}"
        );
    }
    press("tab", cx);
    press("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["london"],
        "apart, the first pin takes its press"
    );
}

/// A zoom while a cluster holds focus regroups it; focus goes to the map, whose arrows still pan.
#[gpui::test]
fn a_zoom_hands_a_focused_clusters_focus_to_the_map(cx: &mut TestAppContext) {
    let (host, cx) = stage(crowd, cx);
    focus_map(cx);
    press("tab", cx);
    press("=", cx);
    assert_eq!(last(&host, cx).zoom, 3.0);
    let before = moves(&host, cx);
    press("left", cx);
    assert_eq!(moves(&host, cx), before + 1, "the map holds focus");
}

const SHAPES: &str = r#"{"type": "FeatureCollection", "features": [
    {"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [
        [[-10, 30], [30, 30], [30, 60], [-10, 60], [-10, 30]],
        [[0, 40], [10, 40], [10, 50], [0, 40]]]}},
    {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[170, 0], [-170, 5]]}},
    {"type": "Feature", "geometry": {"type": "MultiPoint", "coordinates": [[0, 0], [120, -30]]}}
]}"#;

fn layered(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    let features = read_geojson(SHAPES).expect("valid GeoJSON");
    MapView::new("map", MapViewport::new(LatLon::new(20.0, 0.0), 0.5))
        .layer(RouteLine::new([LatLon::new(51.5, -0.1), LatLon::new(40.7, -74.0)]).hue(0))
        .layer(GeoJsonLayer::new(features).hue(3))
        .layer(GeoHeatmap::new(vec![
            (LatLon::new(35.7, 139.7), 1.0),
            (LatLon::new(-33.9, 18.4), 2.0),
        ]))
        .on_viewport(move |view, _, cx| hear(&owner, view, cx))
        .into_any_element()
}

/// A route, areas with a hole, a line over the date line, points and heat paint across the repeated worlds of a view zoomed out, and the map still moves.
#[gpui::test]
fn every_layer_paints_over_repeated_worlds(cx: &mut TestAppContext) {
    let (host, cx) = stage(layered, cx);
    focus_map(cx);
    press("left", cx);
    assert_eq!(moves(&host, cx), 1);
}

#[test]
#[should_panic(expected = "a route of 1 place")]
fn a_route_of_one_place_fails() {
    RouteLine::new([LatLon::new(0.0, 0.0)]);
}
