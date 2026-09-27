use gpui::SharedString;

/// A value in a document: nothing, a flag, a number, a text, or a list or a map of values in their order.
#[derive(Clone, Debug, PartialEq)]
pub enum Datum {
    Null,
    Bool(bool),
    Number(SharedString),
    Text(SharedString),
    List(Vec<Datum>),
    Map(Vec<(SharedString, Datum)>),
}

impl From<&serde_json::Value> for Datum {
    fn from(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Datum::Null,
            serde_json::Value::Bool(on) => Datum::Bool(*on),
            serde_json::Value::Number(number) => Datum::Number(number.to_string().into()),
            serde_json::Value::String(text) => Datum::Text(text.clone().into()),
            serde_json::Value::Array(items) => Datum::List(items.iter().map(Datum::from).collect()),
            serde_json::Value::Object(map) => Datum::Map(
                map.iter()
                    .map(|(key, value)| (key.clone().into(), Datum::from(value)))
                    .collect(),
            ),
        }
    }
}

/// A line that failed to read, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Unread {
    pub line: usize,
    pub why: String,
}

impl std::fmt::Display for Unread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.line, self.why)
    }
}

/// JSON read into a value, keys in their written order.
pub fn read_json(text: &str) -> Result<Datum, Unread> {
    serde_json::from_str::<serde_json::Value>(text)
        .map(|value| Datum::from(&value))
        .map_err(|error| Unread {
            line: error.line(),
            why: error.to_string(),
        })
}

fn scalar(text: &str) -> Datum {
    let text = text.trim();
    let unquoted = |quote: char| {
        text.strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
    };
    if let Some(inner) = unquoted('"').or_else(|| unquoted('\'')) {
        return Datum::Text(inner.to_string().into());
    }
    match text {
        "" | "~" | "null" => Datum::Null,
        "true" => Datum::Bool(true),
        "false" => Datum::Bool(false),
        _ if text.parse::<f64>().is_ok() => Datum::Number(text.to_string().into()),
        _ => Datum::Text(text.to_string().into()),
    }
}

/// Lines of a YAML block: each one's number, indent and words, without comments and blank lines.
fn lines(text: &str) -> Vec<(usize, usize, &str)> {
    text.lines()
        .enumerate()
        .filter_map(|(ix, line)| {
            let words = match line.find(" #") {
                Some(at) => &line[..at],
                None if line.trim_start().starts_with('#') => "",
                None => line,
            };
            let indent = words.len() - words.trim_start().len();
            (!words.trim().is_empty() && words.trim() != "---")
                .then(|| (ix + 1, indent, words.trim_end()))
        })
        .collect()
}

/// The block that starts at `at` and holds the lines at `indent`.
fn block(lines: &[(usize, usize, &str)], at: &mut usize, indent: usize) -> Result<Datum, Unread> {
    let list = lines[*at].2.trim_start().starts_with("- ") || lines[*at].2.trim() == "-";
    if list {
        let mut items = Vec::new();
        while *at < lines.len() && lines[*at].1 == indent {
            let (number, _, words) = lines[*at];
            let Some(rest) = words.trim_start().strip_prefix('-') else {
                return Err(Unread {
                    line: number,
                    why: "a list holds only items starting with -".into(),
                });
            };
            let rest = rest.trim_start();
            *at += 1;
            items.push(if rest.is_empty() {
                nested(lines, at, indent)?
            } else if rest.contains(": ") || rest.ends_with(':') {
                let inner = indent + words.trim_start().len() - rest.len();
                entries(lines, at, inner, Some((number, rest)))?
            } else {
                scalar(rest)
            });
        }
        Ok(Datum::List(items))
    } else {
        entries(lines, at, indent, None)
    }
}

/// What lies under a key or a dash with nothing after it: a deeper block, or nothing.
fn nested(lines: &[(usize, usize, &str)], at: &mut usize, indent: usize) -> Result<Datum, Unread> {
    match lines.get(*at) {
        Some((_, deeper, _)) if *deeper > indent => block(lines, at, *deeper),
        _ => Ok(Datum::Null),
    }
}

/// A map's entries at `indent`, the first one given when it sat after a dash.
fn entries(
    lines: &[(usize, usize, &str)],
    at: &mut usize,
    indent: usize,
    first: Option<(usize, &str)>,
) -> Result<Datum, Unread> {
    let mut map = Vec::new();
    let mut entry = |number: usize, words: &str, at: &mut usize| -> Result<(), Unread> {
        let (key, value) = match words.split_once(": ") {
            Some((key, value)) => (key, value),
            None => match words.strip_suffix(':') {
                Some(key) => (key, ""),
                None => {
                    return Err(Unread {
                        line: number,
                        why: format!("{words:?} is no key and value"),
                    });
                }
            },
        };
        let key = SharedString::from(key.trim().trim_matches('"').to_string());
        let value = if value.trim().is_empty() {
            nested(lines, at, indent)?
        } else {
            scalar(value)
        };
        map.push((key, value));
        Ok(())
    };
    if let Some((number, words)) = first {
        entry(number, words, at)?;
    }
    while *at < lines.len() && lines[*at].1 == indent {
        let (number, _, words) = lines[*at];
        *at += 1;
        entry(number, words.trim_start(), at)?;
    }
    if let Some((number, deeper, _)) = lines.get(*at)
        && *deeper > indent
    {
        return Err(Unread {
            line: *number,
            why: "this line is indented further than its block".into(),
        });
    }
    Ok(Datum::Map(map))
}

/// YAML's block style read into a value: maps, lists and scalars, with comments dropped. Flow style reads as text.
pub fn read_yaml(text: &str) -> Result<Datum, Unread> {
    let lines = lines(text);
    if lines.is_empty() {
        return Ok(Datum::Null);
    }
    let mut at = 0;
    let datum = block(&lines, &mut at, lines[0].1)?;
    match lines.get(at) {
        Some((number, _, _)) => Err(Unread {
            line: *number,
            why: "this line sits outside the document's first block".into(),
        }),
        None => Ok(datum),
    }
}

#[cfg(test)]
mod tests {
    use super::{Datum, read_json, read_yaml};

    fn text(words: &str) -> Datum {
        Datum::Text(words.to_string().into())
    }

    #[test]
    fn json_keeps_its_keys_in_order_and_says_where_it_breaks() {
        let datum = read_json(r#"{"b": 1, "a": [true, null, "x"]}"#).expect("json");
        let Datum::Map(entries) = datum else {
            panic!("a map")
        };
        assert_eq!(entries[0].0, "b");
        assert_eq!(
            entries[1].1,
            Datum::List(vec![Datum::Bool(true), Datum::Null, text("x")])
        );
        assert_eq!(
            read_json("{\n  \"a\": 1,\n}")
                .expect_err("a trailing comma")
                .line,
            3
        );
    }

    #[test]
    fn yaml_reads_nested_maps_lists_and_maps_in_lists() {
        let datum = read_yaml(
            "apiVersion: apps/v1 # the api\nkind: Deployment\nspec:\n  replicas: 3\n  ports:\n    - 80\n    - name: https\n      port: 443\n",
        )
        .expect("yaml");
        let Datum::Map(entries) = datum else {
            panic!("a map")
        };
        assert_eq!(entries[0], ("apiVersion".into(), text("apps/v1")));
        let Datum::Map(spec) = &entries[2].1 else {
            panic!("spec is a map")
        };
        assert_eq!(spec[0], ("replicas".into(), Datum::Number("3".into())));
        let Datum::List(ports) = &spec[1].1 else {
            panic!("ports is a list")
        };
        assert_eq!(ports[0], Datum::Number("80".into()));
        assert_eq!(
            ports[1],
            Datum::Map(vec![
                ("name".into(), text("https")),
                ("port".into(), Datum::Number("443".into()))
            ])
        );
    }

    #[test]
    fn yaml_says_which_line_it_could_not_read() {
        assert_eq!(read_yaml("a: 1\njust words\n").expect_err("no key").line, 2);
        assert_eq!(
            read_yaml("a: 1\n    b: 2\n")
                .expect_err("stray indent")
                .line,
            2
        );
    }
}
