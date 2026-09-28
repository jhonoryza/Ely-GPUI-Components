use std::sync::{Arc, LazyLock};

use gpui::{App, ElementId, IntoElement, RenderOnce, Window};
use isocountry::CountryCode;
use serde_json::Value;

use super::{ChoroplethLayer, ChoroplethMap, Feature, MapView, read_geojson};
use crate::Assets;

/// Natural Earth's 110m countries, read once from the bundle: each keeps its name and, but for two, its ISO 3166 alpha-2 code as `iso`.
pub(crate) static COUNTRIES: LazyLock<Arc<[Feature]>> = LazyLock::new(|| {
    let file = Assets::get("maps/countries.geojson").expect("the world's countries are bundled");
    let text = std::str::from_utf8(&file.data).expect("the countries are UTF-8");
    read_geojson(text)
        .expect("the bundled countries read")
        .into()
});

fn iso(country: &Feature) -> Option<&str> {
    country.properties.get("iso").and_then(Value::as_str)
}

/// The world's countries, washed each by its value keyed by ISO 3166 alpha-2 code, with a legend; Natural Earth is named in the corner unless the map names a source. Fails on a key that is no ISO code; a code with no country at this scale draws nowhere and is logged once.
#[derive(IntoElement)]
pub struct WorldMap {
    id: ElementId,
    map: MapView,
    values: Vec<(String, f64)>,
    hue: usize,
}

impl WorldMap {
    pub fn new(
        id: impl Into<ElementId>,
        map: MapView,
        values: impl IntoIterator<Item = (impl Into<String>, f64)>,
    ) -> Self {
        Self {
            id: id.into(),
            map,
            values: values
                .into_iter()
                .map(|(code, value)| (code.into(), value))
                .collect(),
            hue: 0,
        }
    }

    /// The chart hue the scale runs to.
    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = hue;
        self
    }
}

impl RenderOnce for WorldMap {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        for (code, _) in &self.values {
            let known = CountryCode::for_alpha2(code).is_ok()
                || COUNTRIES.iter().any(|country| iso(country) == Some(code));
            assert!(
                known,
                "world map {:?}: {code} is no ISO 3166 alpha-2 code",
                self.id
            );
        }
        let map = match self.map.attribution.is_some() {
            true => self.map,
            false => self.map.attribution("Countries: Natural Earth"),
        };
        let layer = ChoroplethLayer::new(COUNTRIES.clone(), self.values)
            .key_by("iso")
            .hue(self.hue);
        ChoroplethMap::new(self.id, map, layer)
    }
}

#[cfg(test)]
mod tests {
    use super::{COUNTRIES, iso};

    #[test]
    fn the_bundle_holds_the_worlds_countries_by_code() {
        assert_eq!(COUNTRIES.len(), 177);
        let named = |code: &str| {
            COUNTRIES
                .iter()
                .find(|country| iso(country) == Some(code))
                .and_then(|country| country.properties.get("name"))
                .and_then(|name| name.as_str())
        };
        assert_eq!(
            named("FR"),
            Some("France"),
            "France keeps its code though Natural Earth's ISO_A2 lacks it"
        );
        assert_eq!(named("JP"), Some("Japan"));
        assert_eq!(
            COUNTRIES
                .iter()
                .filter(|country| iso(country).is_none())
                .count(),
            2
        );
    }
}
