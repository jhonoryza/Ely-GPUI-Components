use std::f64::consts::PI;

/// A place on the globe in degrees: latitude north, longitude east.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl LatLon {
    /// Fails past 90° of latitude or 180° of longitude.
    pub fn new(lat: f64, lon: f64) -> Self {
        assert!(
            (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon),
            "no place at {lat}, {lon}"
        );
        Self { lat, lon }
    }
}

/// Web Mercator's last latitude, where its square world ends.
pub const MAX_LAT: f64 = 85.051_128_78;

/// A place on Web Mercator's unit square: x east from 180° west, y south from the top.
pub(crate) fn project(at: LatLon) -> (f64, f64) {
    let phi = at.lat.clamp(-MAX_LAT, MAX_LAT).to_radians();
    let y = (1.0 - (phi.tan() + 1.0 / phi.cos()).ln() / PI) / 2.0;
    ((at.lon + 180.0) / 360.0, y)
}

/// The place at a point of the unit square; x wraps around the globe.
pub(crate) fn unproject((x, y): (f64, f64)) -> LatLon {
    let lat = (PI * (1.0 - 2.0 * y.clamp(0.0, 1.0))).sinh().atan();
    LatLon::new(lat.to_degrees(), x.rem_euclid(1.0) * 360.0 - 180.0)
}

/// A tile of the world at zoom `z`: `x` east and `y` south, each from 0 to 2^z − 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

/// A tile laid in a view: its column before it wraps around the globe, and its box in whole view pixels, so neighbors meet without a seam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Laid {
    pub tile: Tile,
    pub column: i64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Where a map looks: the place at its center, and its zoom, 0 for the world in one tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapViewport {
    pub center: LatLon,
    pub zoom: f64,
}

impl MapViewport {
    /// The farthest out and the nearest in a map zooms.
    pub const ZOOMS: (f64, f64) = (0.0, 19.0);

    pub fn new(center: LatLon, zoom: f64) -> Self {
        let (least, most) = Self::ZOOMS;
        assert!(
            (least..=most).contains(&zoom),
            "zoom {zoom} past {least} to {most}"
        );
        Self { center, zoom }
    }

    /// The world's side in view pixels.
    fn world(&self, tile: f32) -> f64 {
        f64::from(tile) * self.zoom.exp2()
    }

    /// The center in world pixels.
    fn middle(&self, tile: f32) -> (f64, f64) {
        let (x, y) = project(self.center);
        let side = self.world(tile);
        (x * side, y * side)
    }

    /// A place in view pixels from the view's top left: of the globe's copies, the one nearest the center.
    pub(crate) fn to_view(self, at: LatLon, (w, h): (f32, f32), tile: f32) -> (f32, f32) {
        let side = self.world(tile);
        let (cx, cy) = self.middle(tile);
        let (x, y) = project(at);
        let across = (x * side - cx + side / 2.0).rem_euclid(side) - side / 2.0;
        (
            (across + f64::from(w) / 2.0) as f32,
            (y * side - cy + f64::from(h) / 2.0) as f32,
        )
    }

    /// The place at a view point.
    pub(crate) fn to_geo(self, (x, y): (f32, f32), (w, h): (f32, f32), tile: f32) -> LatLon {
        let side = self.world(tile);
        let (cx, cy) = self.middle(tile);
        unproject((
            (cx + f64::from(x) - f64::from(w) / 2.0) / side,
            (cy + f64::from(y) - f64::from(h) / 2.0) / side,
        ))
    }

    /// Moved so the map slides by a view distance.
    pub(crate) fn panned(self, (dx, dy): (f32, f32), size: (f32, f32), tile: f32) -> Self {
        let (cx, cy) = self.middle(tile);
        self.centered((cx - f64::from(dx), cy - f64::from(dy)), size, tile)
    }

    /// Zoomed by `levels` about a view point, whose place stays under it; held inside the zooms.
    pub(crate) fn zoomed(
        self,
        levels: f64,
        about: (f32, f32),
        size: (f32, f32),
        tile: f32,
    ) -> Self {
        let (least, most) = Self::ZOOMS;
        let (x, y) = project(self.to_geo(about, size, tile));
        let next = Self {
            zoom: (self.zoom + levels).clamp(least, most),
            ..self
        };
        let side = next.world(tile);
        let (w, h) = size;
        next.centered(
            (
                x * side - (f64::from(about.0) - f64::from(w) / 2.0),
                y * side - (f64::from(about.1) - f64::from(h) / 2.0),
            ),
            size,
            tile,
        )
    }

    /// Centered on a world pixel; a world as tall as the view or taller keeps the view filled.
    fn centered(self, (x, y): (f64, f64), (_, h): (f32, f32), tile: f32) -> Self {
        let side = self.world(tile);
        let half = f64::from(h) / 2.0;
        let y = if side >= f64::from(h) {
            y.clamp(half, side - half)
        } else {
            side / 2.0
        };
        Self {
            center: unproject((x / side, y / side)),
            ..self
        }
    }

    /// The tiles that cover a view, cut at the nearest zoom within `zooms` and scaled to the view's.
    pub(crate) fn tiles(&self, (w, h): (f32, f32), tile: f32, zooms: (u8, u8)) -> Vec<Laid> {
        if w <= 0.0 || h <= 0.0 {
            return Vec::new();
        }
        let z = (self.zoom.round() as u8).clamp(zooms.0, zooms.1);
        let count = 1i64 << z;
        let side = f64::from(tile) * (self.zoom - f64::from(z)).exp2();
        let (cx, cy) = self.middle(tile);
        let (left, top) = (f64::from(w) / 2.0 - cx, f64::from(h) / 2.0 - cy);
        let span =
            |from: f64, to: f64| (from / side).floor() as i64..=(to / side).ceil() as i64 - 1;
        let edge = |start: f64, ix: i64| (start + ix as f64 * side).round() as f32;
        let rows = span(-top, f64::from(h) - top);
        let (first, last) = ((*rows.start()).max(0), (*rows.end()).min(count - 1));
        span(-left, f64::from(w) - left)
            .flat_map(|column| {
                (first..=last).map(move |row| {
                    let (x, y) = (edge(left, column), edge(top, row));
                    Laid {
                        tile: Tile {
                            z,
                            x: column.rem_euclid(count) as u32,
                            y: row as u32,
                        },
                        column,
                        x,
                        y,
                        w: edge(left, column + 1) - x,
                        h: edge(top, row + 1) - y,
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{LatLon, MAX_LAT, MapViewport, Tile, project, unproject};

    const TILE: f32 = 256.0;
    const VIEW: (f32, f32) = (800.0, 600.0);

    fn near(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn the_square_holds_the_globe_up_to_its_last_latitude() {
        assert_eq!(project(LatLon::new(0.0, 0.0)), (0.5, 0.5));
        let (_, top) = project(LatLon::new(MAX_LAT, -180.0));
        assert!(top.abs() < 1e-9, "the last latitude is the top: {top}");
        assert_eq!(
            project(LatLon::new(90.0, 0.0)),
            project(LatLon::new(MAX_LAT, 0.0))
        );
        let london = LatLon::new(51.5072, -0.1276);
        let back = unproject(project(london));
        assert!((back.lat - london.lat).abs() < 1e-9 && (back.lon - london.lon).abs() < 1e-9);
        assert_eq!(
            unproject((1.25, 0.5)).lon,
            -90.0,
            "x wraps around the globe"
        );
    }

    #[test]
    #[should_panic(expected = "no place at 91, 0")]
    fn a_place_past_the_pole_fails() {
        LatLon::new(91.0, 0.0);
    }

    #[test]
    #[should_panic(expected = "zoom 20 past 0 to 19")]
    fn a_zoom_past_the_nearest_fails() {
        MapViewport::new(LatLon::new(0.0, 0.0), 20.0);
    }

    #[test]
    fn a_place_nearest_the_center_across_the_date_line() {
        let view = MapViewport::new(LatLon::new(0.0, 179.0), 2.0);
        let (x, _) = view.to_view(LatLon::new(0.0, -179.0), VIEW, TILE);
        let degree = TILE * 4.0 / 360.0;
        assert!(
            (x - (400.0 + 2.0 * degree)).abs() < 1e-3,
            "east of the center, not a world away: {x}"
        );
    }

    #[test]
    fn a_pan_slides_the_map_with_the_pointer() {
        let view = MapViewport::new(LatLon::new(20.0, 10.0), 3.0);
        let place = view.to_geo((300.0, 200.0), VIEW, TILE);
        let moved = view.panned((50.0, -30.0), VIEW, TILE);
        assert!(near(moved.to_view(place, VIEW, TILE), (350.0, 170.0)));
    }

    #[test]
    fn a_zoom_keeps_the_place_under_the_pointer() {
        let view = MapViewport::new(LatLon::new(40.0, -74.0), 4.0);
        let about = (620.0, 150.0);
        let place = view.to_geo(about, VIEW, TILE);
        let nearer = view.zoomed(1.5, about, VIEW, TILE);
        assert_eq!(nearer.zoom, 5.5);
        assert!(near(nearer.to_view(place, VIEW, TILE), about));
        assert_eq!(
            view.zoomed(99.0, about, VIEW, TILE).zoom,
            MapViewport::ZOOMS.1
        );
    }

    #[test]
    fn the_world_fills_the_view_it_is_tall_enough_for() {
        let north = MapViewport::new(LatLon::new(80.0, 0.0), 2.0).panned((0.0, 0.0), VIEW, TILE);
        let (_, top) = north.to_view(LatLon::new(MAX_LAT, 0.0), VIEW, TILE);
        assert!(
            top.abs() < 1e-3,
            "the top edge stays at the view's top: {top}"
        );
        let whole = MapViewport::new(LatLon::new(60.0, 0.0), 0.0).panned((0.0, 40.0), VIEW, TILE);
        assert!(
            whole.center.lat.abs() < 1e-9,
            "a world shorter than the view sits in its middle"
        );
    }

    #[test]
    fn tiles_cover_the_view_and_wrap_around_the_globe() {
        let view = MapViewport::new(LatLon::new(0.0, 0.0), 1.0);
        let laid = view.tiles(VIEW, TILE, (0, 19));
        let columns: Vec<i64> = laid.iter().map(|laid| laid.column).collect();
        assert_eq!(laid.len(), 8, "two rows of four columns");
        assert_eq!(columns.iter().min(), Some(&-1));
        assert_eq!(columns.iter().max(), Some(&2));
        let west = laid
            .iter()
            .find(|laid| laid.column == -1)
            .expect("a copy past the west edge");
        assert_eq!(west.tile, Tile { z: 1, x: 1, y: 0 });
        assert_eq!(
            (west.x, west.y, west.w, west.h),
            (-112.0, 44.0, 256.0, 256.0)
        );
        assert!(
            laid.iter()
                .all(|laid| laid.x < VIEW.0 && laid.x + laid.w > 0.0)
        );
    }

    #[test]
    fn tiles_meet_on_whole_pixels_at_a_fractional_zoom() {
        let laid = MapViewport::new(LatLon::new(12.3, 45.6), 2.37).tiles(VIEW, TILE, (0, 19));
        for tile in &laid {
            assert_eq!(
                (tile.x.fract(), tile.y.fract(), tile.w.fract()),
                (0.0, 0.0, 0.0)
            );
            let next = |column: i64, row: u32| {
                laid.iter()
                    .find(|each| each.column == column && each.tile.y == row)
            };
            if let Some(east) = next(tile.column + 1, tile.tile.y) {
                assert_eq!(
                    tile.x + tile.w,
                    east.x,
                    "no seam between {tile:?} and {east:?}"
                );
            }
            if let Some(south) = next(tile.column, tile.tile.y + 1) {
                assert_eq!(tile.y + tile.h, south.y);
            }
        }
    }

    #[test]
    fn tiles_past_the_hosts_zooms_scale() {
        let view = MapViewport::new(LatLon::new(0.0, 0.0), 5.0);
        let laid = view.tiles(VIEW, TILE, (0, 3));
        assert!(
            laid.iter()
                .all(|laid| laid.tile.z == 3 && laid.w == 1024.0 && laid.h == 1024.0)
        );
        let far = MapViewport::new(LatLon::new(0.0, 0.0), 2.4).tiles(VIEW, TILE, (0, 19));
        assert!(
            far.iter().all(|laid| laid.tile.z == 2),
            "the nearest whole zoom"
        );
        assert!(view.tiles((0.0, 600.0), TILE, (0, 19)).is_empty());
    }
}
