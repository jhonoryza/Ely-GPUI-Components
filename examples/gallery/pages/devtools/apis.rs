use ely_gpui_component::{
    devtools::{
        ApiRequest, ApiRequestBuilder, ApiResponse, CollectionTree, Direction, Environment,
        EnvironmentSelector, GraphQLExplorer, Method, ResponseViewer, Saved, SchemaType,
        SocketMessage, SocketState, TypeKind, WebSocketConsole,
    },
    editor::CodeEditor,
    forms::TextInput,
    theme::{ActiveTheme, Radius},
};
use gpui::{
    App, AppContext as _, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::Timestamp;

use crate::ui::{change, keep, section};

fn environments() -> Vec<Environment> {
    let env = |key: &str, name: &str, host: &str| Environment {
        key: key.to_string().into(),
        name: name.to_string().into(),
        variables: vec![
            ("host".into(), host.to_string().into()),
            ("token".into(), "sk_live_4f9a2c".into()),
        ],
        secrets: vec!["token".into()],
    };
    vec![
        env("local", "Local", "localhost:8080"),
        env("staging", "Staging", "staging.api.example.com"),
        env("production", "Production", "api.example.com"),
    ]
}

/// The API demo's request, its environment, the answer and the request picked.
struct Desk {
    request: ApiRequest,
    environment: SharedString,
    response: Option<ApiResponse>,
    picked: Vec<SharedString>,
}

impl Desk {
    fn new() -> Self {
        let mut request = ApiRequest::new("list-orders", Method::Get, "https://{{host}}/v1/orders");
        request.params = vec![
            ("status".into(), "open".into()),
            ("limit".into(), "20".into()),
        ];
        request.headers = vec![("Accept".into(), "application/json".into())];
        Self {
            request,
            environment: "staging".into(),
            response: Some(answer()),
            picked: vec!["list-orders".into()],
        }
    }
}

fn answer() -> ApiResponse {
    ApiResponse {
        status: 200,
        took_ms: 142,
        size: 1_380,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("cache-control".into(), "no-store".into()),
            ("x-request-id".into(), "a1f3-77c2".into()),
        ],
        body: r#"{"orders":[{"id":1009,"total":89.0,"status":"open"},{"id":1010,"total":26.0,"status":"open"}],"next":null}"#.into(),
    }
}

pub fn requests(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let desk = keep("devtools-api", Desk::new, window, cx);
    let now = desk.read(cx);
    let (request, environment, response, picked) = (
        now.request.clone(),
        now.environment.clone(),
        now.response.clone(),
        now.picked.clone(),
    );
    let variables = environments()
        .into_iter()
        .find(|each| each.key == environment)
        .expect("a listed environment")
        .variables;
    let [sent, chose, opened] = [(); 3].map(|_| desk.clone());
    let collection = [
        Saved::folder(
            "orders",
            "Orders",
            [
                Saved::request("list-orders", "List orders", Method::Get),
                Saved::request("create-order", "Create an order", Method::Post),
                Saved::request("cancel-order", "Cancel an order", Method::Delete),
            ],
        ),
        Saved::folder(
            "users",
            "Users",
            [
                Saved::request("me", "Who am I", Method::Get),
                Saved::request("rename", "Rename", Method::Patch),
            ],
        ),
    ];
    let theme = cx.theme();
    section(
        "ApiRequestBuilder · ResponseViewer · HeadersTable · QueryParamsEditor → forms::KeyValueInput · EnvironmentSelector · CollectionTree",
        "Saved requests in folders; the one open, built with its method and address, its query's pairs, headers, body and sign-in, and the address that goes out with the environment's variables filled in. Send brings back an answer: its status, time and size, then its body or its headers.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div()
                    .w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        EnvironmentSelector::new("devtools-environments", environments(), environment)
                            .on_change(move |key, _, cx| change(&chose, cx, |desk| desk.environment = key.clone())),
                    )
                    .child(
                        div()
                            .h(px(220.))
                            .p_1()
                            .rounded(theme.radius(Radius::Lg))
                            .border_1()
                            .border_color(theme.colors.border)
                            .child(
                                CollectionTree::new("devtools-collection", collection)
                                    .open(["orders"])
                                    .selected(picked)
                                    .on_open(move |key, _, cx| change(&opened, cx, |desk| desk.picked = vec![key.clone()])),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(560.))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        ApiRequestBuilder::new("devtools-request", request)
                            .variables(variables)
                            .on_send(move |_, _, cx| change(&sent, cx, |desk| desk.response = Some(answer()))),
                    )
                    .children(response.map(|response| ResponseViewer::new("devtools-response", response))),
            ),
    )
}

/// The socket demo's state, messages and draft.
struct Line {
    state: SocketState,
    messages: Vec<SocketMessage>,
    draft: gpui::Entity<TextInput>,
    now: Timestamp,
}

pub fn socket(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let line = window.use_keyed_state("devtools-socket", cx, |window, cx| {
        let now: Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
        let at = |seconds: i64| now - jiff::SignedDuration::from_secs(seconds);
        let message = |direction, text: &str, seconds| SocketMessage {
            direction,
            text: text.to_string().into(),
            at: at(seconds),
        };
        Line {
            state: SocketState::Open,
            messages: vec![
                message(
                    Direction::Note,
                    "Connected to wss://stream.example.com/orders",
                    600,
                ),
                message(
                    Direction::Sent,
                    r#"{"type":"subscribe","channel":"orders"}"#,
                    598,
                ),
                message(
                    Direction::Received,
                    r#"{"type":"order","id":1011,"total":42.5}"#,
                    120,
                ),
            ],
            draft: cx.new(|cx| TextInput::new(window, cx).placeholder("A message to send")),
            now,
        }
    });
    let now = line.read(cx);
    let (state, messages, draft, clock) =
        (now.state, now.messages.clone(), now.draft.clone(), now.now);
    let (sent, toggled) = (line.clone(), line.clone());
    section(
        "WebSocketConsole",
        "A socket's conversation: where it stands, each message marked sent or received with its time, JSON indented, and a field that sends on Enter.",
        cx,
    )
    .child(
        div().w(px(620.)).child(
            WebSocketConsole::new("devtools-socket", "wss://stream.example.com/orders", state, messages, &draft, clock)
                .on_send(move |text, _, cx| {
                    sent.update(cx, |line, cx| {
                        let at = line.now;
                        line.messages.push(SocketMessage { direction: Direction::Sent, text, at });
                        cx.notify();
                    })
                })
                .on_toggle(move |_, cx| {
                    toggled.update(cx, |line, cx| {
                        line.state = match line.state {
                            SocketState::Closed => SocketState::Open,
                            _ => SocketState::Closed,
                        };
                        cx.notify();
                    })
                }),
        ),
    )
}

/// The GraphQL demo's editors, find field and answer.
struct Graph {
    query: gpui::Entity<CodeEditor>,
    variables: gpui::Entity<CodeEditor>,
    search: gpui::Entity<TextInput>,
    answer: Option<SharedString>,
}

/// What the demo's query brings back.
const ANSWER: &str =
    r#"{"data":{"order":{"id":"1009","total":89.0,"customer":{"name":"Ada Lovelace"}}}}"#;

pub fn graphql(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let graph = window.use_keyed_state("devtools-graphql", cx, |window, cx| Graph {
        query: cx.new(|cx| CodeEditor::new("query Order($id: ID!) {\n  order(id: $id) {\n    id\n    total\n    customer { name }\n  }\n}", window, cx).language("GraphQL")),
        variables: cx.new(|cx| CodeEditor::new(r#"{"id": "1009"}"#, window, cx).language("JSON")),
        search: cx.new(|cx| TextInput::new(window, cx).placeholder("Find in the answer")),
        answer: Some(ANSWER.into()),
    });
    let now = graph.read(cx);
    let (query, variables, search, answer) = (
        now.query.clone(),
        now.variables.clone(),
        now.search.clone(),
        now.answer.clone(),
    );
    let ran = graph.clone();
    let types = vec![
        SchemaType {
            name: "Query".into(),
            kind: TypeKind::Object,
            fields: vec![
                ("order".into(), "Order".into()),
                ("orders".into(), "[Order!]!".into()),
            ],
        },
        SchemaType {
            name: "Order".into(),
            kind: TypeKind::Object,
            fields: vec![
                ("id".into(), "ID!".into()),
                ("total".into(), "Float!".into()),
                ("status".into(), "Status!".into()),
                ("customer".into(), "Customer!".into()),
            ],
        },
        SchemaType {
            name: "Customer".into(),
            kind: TypeKind::Object,
            fields: vec![("name".into(), "String!".into())],
        },
        SchemaType {
            name: "Status".into(),
            kind: TypeKind::Enum,
            fields: vec![("OPEN".into(), "".into()), ("SHIPPED".into(), "".into())],
        },
    ];
    let mut explorer = GraphQLExplorer::new("devtools-graphql", types, &query, &variables, &search)
        .on_run(move |_, _, _, cx| change(&ran, cx, |graph| graph.answer = Some(ANSWER.into())));
    if let Some(answer) = answer {
        explorer = explorer.answer(answer);
    }
    section(
        "GraphQLExplorer",
        "A schema's types and fields beside a query and its variables; Run brings back the answer as a tree.",
        cx,
    )
    .child(div().w(px(860.)).child(explorer))
}
