use gpui::{AnyElement, Entity, IntoElement, TestAppContext, VisualTestContext};

use super::{
    Stage, said, say, settle, stage,
    tools::{drag, layer, pair, press},
};
use crate::canvas::{
    Board, Edge, MindMap, Node, NodeGraph, Port, Tool, Topic, Viewport, Whiteboard,
};

fn tap(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

#[gpui::test]
fn the_connector_links_the_shape_pressed_to_the_one_released_on(cx: &mut TestAppContext) {
    let (host, cx) = stage(|_, owner| layer(Tool::Connector, &[], owner), cx);
    drag((50.0, 50.0), (250.0, 60.0), cx);
    drag((50.0, 50.0), (150.0, 200.0), cx);
    assert_eq!(
        said(&host, cx),
        ["link a b"],
        "a release on nothing links nothing"
    );
}

#[gpui::test]
fn a_double_press_writes_on_a_shape_and_enter_keeps_the_words(cx: &mut TestAppContext) {
    let (host, cx) = stage(|_, owner| layer(Tool::Select, &["a"], owner), cx);
    press(50.0, 50.0, 1, cx);
    press(50.0, 50.0, 2, cx);
    cx.simulate_input("Start");
    tap("enter", cx);
    assert_eq!(said(&host, cx), ["text a Start"]);
}

fn nodes() -> Vec<Node> {
    vec![
        Node::new("load", "Load", (0.0, 0.0)).output(Port::new("image", "Image", "image")),
        Node::new("blur", "Blur", (300.0, 0.0))
            .input(Port::new("image", "Image", "image"))
            .output(Port::new("image", "Image", "image")),
    ]
}

fn graph(wired: bool, owner: Entity<Stage>) -> AnyElement {
    let (joined, parted) = (owner.clone(), owner);
    let edges = wired.then(|| Edge::new(("load", "image"), ("blur", "image")));
    NodeGraph::new("graph", Viewport::new(0.0, 0.0, 1.0), nodes(), edges)
        .on_connect(move |edge, _, cx| {
            say(
                &joined,
                format!(
                    "connect {}.{} {}.{}",
                    edge.from.0, edge.from.1, edge.to.0, edge.to.1
                ),
                cx,
            )
        })
        .on_disconnect(move |edge, _, cx| {
            say(
                &parted,
                format!("disconnect {}.{}", edge.to.0, edge.to.1),
                cx,
            )
        })
        .into_any_element()
}

/// Sockets sit on the node's sides a title and half a row down: 32 + 14.
#[gpui::test]
fn a_drag_from_an_output_wires_it_to_an_input(cx: &mut TestAppContext) {
    let (host, cx) = stage(|_, owner| graph(false, owner), cx);
    drag((200.0, 46.0), (301.0, 47.0), cx);
    assert_eq!(said(&host, cx), ["connect load.image blur.image"]);
}

#[gpui::test]
fn a_press_on_a_wired_input_lifts_its_wire(cx: &mut TestAppContext) {
    let (host, cx) = stage(|_, owner| graph(true, owner), cx);
    drag((300.0, 46.0), (250.0, 250.0), cx);
    assert_eq!(said(&host, cx), ["disconnect blur.image"]);
}

fn map(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    let root = Topic::new("root", "Launch")
        .child(Topic::new("a", "Site"))
        .child(Topic::new("b", "Press"))
        .child(Topic::new("c", "Budget"));
    let [picked, added, removed] = [(); 3].map(|_| owner.clone());
    MindMap::new("map", root, Viewport::new(-60.0, -150.0, 1.0))
        .selected(Some("a"))
        .on_select(move |key, _, cx| say(&picked, format!("select {key:?}"), cx))
        .on_add(move |key, _, cx| say(&added, format!("add under {key}"), cx))
        .on_remove(move |key, _, cx| say(&removed, format!("remove {key}"), cx))
        .into_any_element()
}

#[gpui::test]
fn keys_on_a_mind_map_ask_for_children_siblings_and_removals(cx: &mut TestAppContext) {
    let (host, cx) = stage(map, cx);
    press(276.0, 126.0, 1, cx);
    for key in ["tab", "enter", "down", "backspace"] {
        tap(key, cx);
    }
    assert_eq!(
        said(&host, cx),
        [
            "select Some(\"a\")",
            "add under a",
            "add under root",
            "select Some(\"b\")",
            "remove a",
        ],
        "a press selects and focuses; the keys act on the owner's selection"
    );
}

fn board(_: &Stage, owner: Entity<Stage>) -> AnyElement {
    Whiteboard::new(
        "board",
        Board {
            shapes: pair(),
            links: Vec::new(),
        },
    )
    .on_change(move |board, _, cx| {
        let keys: Vec<String> = board
            .shapes
            .iter()
            .map(|shape| shape.key.to_string())
            .collect();
        say(&owner, format!("shapes {keys:?}"), cx)
    })
    .into_any_element()
}

#[gpui::test]
fn backspace_takes_the_selection_off_the_board(cx: &mut TestAppContext) {
    let (host, cx) = stage(board, cx);
    press(250.0, 50.0, 1, cx);
    tap("backspace", cx);
    assert_eq!(said(&host, cx), ["shapes [\"a\"]"]);
}
