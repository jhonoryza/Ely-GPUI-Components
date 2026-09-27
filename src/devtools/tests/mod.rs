use gpui::{
    AnyElement, Context, Entity, IntoElement, KeyBinding, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, px,
};

use super::{Connection, ConnectionManager, Engine, Field, TableStructureEditor};
use crate::{forms, primitives::FocusNext, theme::Theme};

/// A view that shows one tool and keeps the words it heard.
struct Bench {
    part: fn(Entity<Bench>) -> AnyElement,
    said: Vec<String>,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(640.0)).child((self.part)(cx.entity()))
    }
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

fn bench(
    part: fn(Entity<Bench>) -> AnyElement,
    cx: &mut TestAppContext,
) -> (Entity<Bench>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
        forms::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
    let (host, cx) = cx.add_window_view(|_, _| Bench {
        part,
        said: Vec::new(),
    });
    settle(cx);
    (host, cx)
}

fn say(owner: &Entity<Bench>, words: String, cx: &mut gpui::App) {
    owner.update(cx, |bench, cx| {
        bench.said.push(words);
        cx.notify();
    });
}

fn said(host: &Entity<Bench>, cx: &mut VisualTestContext) -> Vec<String> {
    host.read_with(cx, |bench, _| bench.said.clone())
}

fn tab(stops: usize, cx: &mut VisualTestContext) {
    cx.update(|window, _| {
        window.blur();
        (0..stops).for_each(|_| window.focus_next());
    });
    settle(cx);
}

fn tap(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

fn manager(owner: Entity<Bench>) -> AnyElement {
    let [opened, edited, picked] = [(); 3].map(|_| owner.clone());
    ConnectionManager::new(
        "connections",
        [
            Connection::new("local", "Local", Engine::Postgres),
            Connection::new("cache", "Cache", Engine::Redis),
        ],
    )
    .on_open(move |key, _, cx| say(&opened, format!("open {key}"), cx))
    .on_edit(move |key, _, cx| say(&edited, format!("edit {key}"), cx))
    .on_select(move |key, _, cx| say(&picked, format!("select {key}"), cx))
    .into_any_element()
}

/// Stops run: New, the list, then each row's Edit and Remove.
#[gpui::test]
fn enter_opens_a_connection_and_a_rows_button_leaves_the_row_alone(cx: &mut TestAppContext) {
    let (host, cx) = bench(manager, cx);
    tab(2, cx);
    tap("enter", cx);
    tab(3, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["open local", "edit local"]);
}

fn structure(owner: Entity<Bench>) -> AnyElement {
    TableStructureEditor::new("structure", [Field::new("id", "int").primary()])
        .on_change(move |fields, _, cx| {
            let names: Vec<String> = fields
                .iter()
                .map(|field| format!("{}:{}", field.name, field.ty))
                .collect();
            say(&owner, names.join(" "), cx)
        })
        .into_any_element()
}

/// A row's stops: name, type, empty, key, default and remove; Add field comes after.
#[gpui::test]
fn add_field_appends_a_free_name(cx: &mut TestAppContext) {
    let (host, cx) = bench(structure, cx);
    tab(7, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["id:int field_1:text"]);
}

#[test]
fn an_address_names_the_host_or_the_file() {
    let mut local = Connection::new("local", "Local", Engine::Postgres);
    local.database = "shop".into();
    assert_eq!(local.address(), "localhost:5432/shop");
    assert_eq!(
        Connection::new("cache", "Cache", Engine::Redis).address(),
        "localhost:6379"
    );
    let mut file = Connection::new("file", "File", Engine::Sqlite);
    file.database = "app.db".into();
    assert_eq!(file.address(), "app.db");
}
