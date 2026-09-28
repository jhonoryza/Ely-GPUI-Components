use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Hsla, IntoElement, PathBuilder, Pixels, Point, Styled, Window, canvas,
    fill, size,
};

use super::{
    LatLon, MapViewport,
    geojson::{Feature, Geometry},
    heat::density,
};
use crate::{
    canvas::{at, finish, outline},
    theme::{ActiveTheme, Palette},
};

/// What a layer paints on: where the map looks, the view's size and top left, the tile's side, and the theme's inks and measures in pixels.
struct Paper {
    view: MapViewport,
    size: (f32, f32),
    origin: Point<Pixels>,
    tile: f32,
    colors: Palette,
    route: Pixels,
    line: Pixels,
    dot: f32,
    cell: f32,
    reach: f32,
}

impl Paper {
    fn ink(&self, hue: Option<usize>, owner: &str) -> Hsla {
        match hue {
            Some(hue) => self.colors.hue(hue, owner),
            None => self.colors.accent,
        }
    }

    /// The shifts by whole worlds that bring points spanning `x` into view.
    fn copies(&self, points: &[(f32, f32)], pad: f32) -> Vec<f32> {
        let least = points
            .iter()
            .map(|point| point.0)
            .fold(f32::INFINITY, f32::min);
        let most = points
            .iter()
            .map(|point| point.0)
            .fold(f32::NEG_INFINITY, f32::max);
        self.view
            .copies((least - pad, most + pad), self.size.0, self.tile)
    }

    fn stroke(
        &self,
        points: &[(f32, f32)],
        shift: f32,
        width: Pixels,
        ink: Hsla,
        window: &mut Window,
    ) {
        let shifted: Vec<_> = points.iter().map(|(x, y)| (x + shift, *y)).collect();
        finish(outline(&shifted, self.origin, width, false), ink, window);
    }

    fn dot(&self, place: LatLon, ink: Hsla, window: &mut Window) {
        let (x, y) = self.view.to_view(place, self.size, self.tile);
        let half = self.dot / 2.0;
        for shift in self.copies(&[(x, y)], half) {
            let corner = at(self.origin, (x + shift - half, y - half));
            let side = Pixels::from(self.dot);
            window.paint_quad(
                fill(Bounds::new(corner, size(side, side)), ink).corner_radii(side / 2.0),
            );
        }
    }

    /// An area's rings washed as one path, so holes stay open, and outlined.
    fn area(&self, rings: &[Vec<LatLon>], ink: Hsla, window: &mut Window) {
        let runs: Vec<Vec<(f32, f32)>> = rings
            .iter()
            .map(|ring| self.view.run(ring, self.size, self.tile))
            .collect();
        let Some(outer) = runs.first() else {
            return;
        };
        for shift in self.copies(outer, 0.0) {
            let mut path = PathBuilder::fill();
            for run in &runs {
                let mut points = run.iter().map(|(x, y)| at(self.origin, (x + shift, *y)));
                if let Some(first) = points.next() {
                    path.move_to(first);
                    points.for_each(|next| path.line_to(next));
                    path.close();
                }
            }
            finish(path, ink.alpha(0.25), window);
            for run in &runs {
                self.stroke(run, shift, self.line, ink, window);
            }
        }
    }

    fn line(&self, places: &[LatLon], width: Pixels, ink: Hsla, window: &mut Window) {
        let run = self.view.run(places, self.size, self.tile);
        for shift in self.copies(&run, 0.0) {
            self.stroke(&run, shift, width, ink, window);
        }
    }

    fn geometry(&self, geometry: &Geometry, ink: Hsla, window: &mut Window) {
        match geometry {
            Geometry::Point(place) => self.dot(*place, ink, window),
            Geometry::MultiPoint(places) => places
                .iter()
                .for_each(|place| self.dot(*place, ink, window)),
            Geometry::LineString(places) => self.line(places, self.line, ink, window),
            Geometry::MultiLineString(lines) => lines
                .iter()
                .for_each(|places| self.line(places, self.line, ink, window)),
            Geometry::Polygon(rings) => self.area(rings, ink, window),
            Geometry::MultiPolygon(polygons) => polygons
                .iter()
                .for_each(|rings| self.area(rings, ink, window)),
            Geometry::Collection(parts) => parts
                .iter()
                .for_each(|part| self.geometry(part, ink, window)),
        }
    }
}

/// Something the map draws over its tiles and under its pins.
#[derive(Clone)]
pub enum MapLayer {
    Route(RouteLine),
    Features(GeoJsonLayer),
    Heat(GeoHeatmap),
}

impl MapLayer {
    fn paint(&self, paper: &Paper, window: &mut Window) {
        match self {
            MapLayer::Route(route) => {
                let ink = paper.ink(route.hue, "route");
                paper.line(&route.places, paper.route, ink, window);
            }
            MapLayer::Features(layer) => {
                let ink = paper.ink(layer.hue, "features");
                for feature in layer.features.iter() {
                    if let Some(geometry) = &feature.geometry {
                        paper.geometry(geometry, ink, window);
                    }
                }
            }
            MapLayer::Heat(heat) => {
                let ink = paper.ink(heat.hue, "heat");
                let mut points = Vec::new();
                for (place, weight) in heat.points.iter() {
                    let (x, y) = paper.view.to_view(*place, paper.size, paper.tile);
                    for shift in paper.copies(&[(x, y)], paper.reach) {
                        points.push(((x + shift, y), *weight));
                    }
                }
                let heat = density(&points, paper.size, paper.cell, paper.reach);
                for row in 0..heat.rows {
                    for column in 0..heat.columns {
                        let value = heat.at(column, row);
                        if value > 0.01 {
                            let corner = (column as f32 * heat.cell, row as f32 * heat.cell);
                            let side = Pixels::from(heat.cell);
                            let bounds = Bounds::new(at(paper.origin, corner), size(side, side));
                            window.paint_quad(fill(bounds, ink.alpha(value * 0.7)));
                        }
                    }
                }
            }
        }
    }
}

/// A line along places in order, as a route runs; it crosses the date line the short way.
#[derive(Clone)]
pub struct RouteLine {
    places: Vec<LatLon>,
    hue: Option<usize>,
}

impl RouteLine {
    /// Fails with fewer than two places.
    pub fn new(places: impl IntoIterator<Item = LatLon>) -> Self {
        let places: Vec<LatLon> = places.into_iter().collect();
        assert!(places.len() >= 2, "a route of {} place", places.len());
        Self { places, hue: None }
    }

    /// The chart hue it wears; the accent without one.
    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = Some(hue);
        self
    }
}

/// GeoJSON features in one hue: areas washed and outlined with their holes open, lines stroked, points dotted.
#[derive(Clone)]
pub struct GeoJsonLayer {
    features: Rc<[Feature]>,
    hue: Option<usize>,
}

impl GeoJsonLayer {
    pub fn new(features: impl Into<Rc<[Feature]>>) -> Self {
        Self {
            features: features.into(),
            hue: None,
        }
    }

    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = Some(hue);
        self
    }
}

/// Weighted places as heat, hotter where they crowd, scaled to the densest spot in view.
#[derive(Clone)]
pub struct GeoHeatmap {
    points: Rc<[(LatLon, f32)]>,
    hue: Option<usize>,
}

impl GeoHeatmap {
    pub fn new(points: impl Into<Rc<[(LatLon, f32)]>>) -> Self {
        Self {
            points: points.into(),
            hue: None,
        }
    }

    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = Some(hue);
        self
    }
}

impl From<RouteLine> for MapLayer {
    fn from(route: RouteLine) -> Self {
        MapLayer::Route(route)
    }
}

impl From<GeoJsonLayer> for MapLayer {
    fn from(layer: GeoJsonLayer) -> Self {
        MapLayer::Features(layer)
    }
}

impl From<GeoHeatmap> for MapLayer {
    fn from(heat: GeoHeatmap) -> Self {
        MapLayer::Heat(heat)
    }
}

/// A canvas that paints `layers` in order over a view of `size`; none without layers.
pub(crate) fn painted(
    layers: Vec<MapLayer>,
    view: MapViewport,
    size: (f32, f32),
    tile: f32,
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    if layers.is_empty() {
        return None;
    }
    let theme = cx.theme();
    let (sizes, rem) = (theme.maps(), window.rem_size());
    let (colors, pixels) = (theme.colors.clone(), |rems: gpui::Rems| rems.to_pixels(rem));
    let (route, line, dot) = (
        pixels(sizes.route),
        pixels(sizes.line),
        f32::from(pixels(sizes.dot)),
    );
    let (cell, reach) = (
        f32::from(pixels(sizes.heat_cell)),
        f32::from(pixels(sizes.heat_reach)),
    );
    Some(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let paper = Paper {
                    view,
                    size,
                    origin: bounds.origin,
                    tile,
                    colors,
                    route,
                    line,
                    dot,
                    cell,
                    reach,
                };
                layers.iter().for_each(|layer| layer.paint(&paper, window));
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element(),
    )
}
