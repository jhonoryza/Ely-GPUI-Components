use gpui::{
    AnyElement, App, AppContext as _, Entity, IntoElement, ParentElement, Styled, TestAppContext,
    Window, div, px,
};

use super::{Bench, bench, said, say, settle, tab, tap};
use crate::{
    devtools::{
        ApiRequest, ApiRequestBuilder, Auth, CollectionTree, GraphQLExplorer, Method, Saved,
        SocketState, WebSocketConsole,
    },
    editor::CodeEditor,
    forms::TextInput,
};

fn builder(_: &Bench, _: &mut Window, _: &mut App, owner: Entity<Bench>) -> AnyElement {
    let mut request = ApiRequest::new("r", Method::Get, "https://{{host}}/items");
    request.params = vec![("q".into(), "red shoes".into())];
    request.auth = Auth::Basic {
        user: "ada".into(),
        password: "pw".into(),
    };
    ApiRequestBuilder::new("builder", request)
        .variables([("host", "api.example.com")])
        .on_send(move |request, _, cx| {
            say(
                &owner,
                format!(
                    "{} {} {:?}",
                    request.method.name(),
                    request.address(),
                    request.auth
                ),
                cx,
            )
        })
        .into_any_element()
}

/// Stops: the method, the address, then Send.
#[gpui::test]
fn send_hands_on_the_request_with_its_pairs(cx: &mut TestAppContext) {
    let (host, cx) = bench(builder, cx);
    tab(3, cx);
    tap("space", cx);
    assert_eq!(
        said(&host, cx),
        [r#"GET https://{{host}}/items?q=red%20shoes Basic { user: "ada", password: "pw" }"#]
    );
}

fn console(bench: &Bench, window: &mut Window, cx: &mut App, owner: Entity<Bench>) -> AnyElement {
    let draft = window.use_keyed_state("draft", cx, TextInput::new);
    let now: jiff::Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
    let state = match bench.choice {
        0 => SocketState::Open,
        _ => SocketState::Closed,
    };
    let toggled = owner.clone();
    WebSocketConsole::new("socket", "wss://example.com", state, [], &draft, now)
        .on_send(move |text, _, cx| say(&owner, format!("send {text}"), cx))
        .on_toggle(move |_, cx| say(&toggled, "toggle".into(), cx))
        .into_any_element()
}

/// Stops: Disconnect, the message field, then Send once it holds words.
#[gpui::test]
fn enter_in_the_field_sends_and_empties_it(cx: &mut TestAppContext) {
    let (host, cx) = bench(console, cx);
    tab(2, cx);
    cx.simulate_input("ping");
    tap("enter", cx);
    tap("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["send ping"],
        "an empty field sends nothing"
    );
}

/// Stops while closed: Connect, then the field; Send rests, so a third Tab wraps to Connect.
#[gpui::test]
fn a_closed_socket_sends_nothing_and_connect_asks_the_owner(cx: &mut TestAppContext) {
    let (host, cx) = bench(console, cx);
    host.update(cx, |bench, cx| {
        bench.choice = 1;
        cx.notify();
    });
    settle(cx);
    tab(2, cx);
    cx.simulate_input("ping");
    tap("enter", cx);
    tab(3, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["toggle"]);
}

fn saved(_: &Bench, _: &mut Window, _: &mut App, owner: Entity<Bench>) -> AnyElement {
    let list = Saved::Request {
        key: "list".into(),
        name: "List orders".into(),
        method: Method::Get,
    };
    let orders = Saved::Folder {
        key: "orders".into(),
        name: "Orders".into(),
        items: vec![list],
    };
    div()
        .h(px(200.0))
        .child(
            CollectionTree::new("saved", [orders])
                .open(["orders"])
                .on_open(move |key, _, cx| say(&owner, format!("open {key}"), cx)),
        )
        .into_any_element()
}

/// Stops: the tree, its cursor on the folder.
#[gpui::test]
fn enter_on_a_saved_request_opens_it(cx: &mut TestAppContext) {
    let (host, cx) = bench(saved, cx);
    tab(1, cx);
    tap("down", cx);
    tap("enter", cx);
    assert_eq!(said(&host, cx), ["open list"]);
}

fn graph(bench: &Bench, window: &mut Window, cx: &mut App, owner: Entity<Bench>) -> AnyElement {
    let (Some(variables), query) = (
        bench.editor.clone(),
        window.use_keyed_state("query", cx, |window, cx| {
            CodeEditor::new("{ a }", window, cx)
        }),
    ) else {
        return div().into_any_element();
    };
    let search = window.use_keyed_state("search", cx, TextInput::new);
    GraphQLExplorer::new("graph", [], &query, &variables, &search)
        .on_run(move |query, variables, _, cx| say(&owner, format!("{query} {variables}"), cx))
        .into_any_element()
}

/// Stops: the schema, the query, the variables, then Run while they read.
#[gpui::test]
fn run_waits_for_variables_that_read(cx: &mut TestAppContext) {
    let (host, cx) = bench(graph, cx);
    let variables = cx.update(|window, cx| cx.new(|cx| CodeEditor::new("{", window, cx)));
    host.update(cx, |bench, cx| {
        bench.editor = Some(variables.clone());
        cx.notify();
    });
    settle(cx);
    tab(4, cx);
    tap("space", cx);
    assert!(
        said(&host, cx).is_empty(),
        "variables that do not read keep Run at rest"
    );
    variables.update(cx, |editor, cx| {
        editor.set_text(r#"{"id": 1}"#.to_string(), cx)
    });
    settle(cx);
    tab(4, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), [r#"{ a } {"id": 1}"#]);
}
