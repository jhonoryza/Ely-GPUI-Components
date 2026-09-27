use std::{cell::RefCell, panic::Location};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Position, Style, Window, size,
};

/// What was raised while a layer prepaints its content, each with where it sits and its priority.
type Queue = Vec<(AnyElement, Point<Pixels>, usize)>;

thread_local! {
    static LAYERS: RefCell<Vec<Queue>> = const { RefCell::new(Vec::new()) };
}

fn nested() -> bool {
    LAYERS.with_borrow(|layers| !layers.is_empty())
}

/// Draws `child` over the page, as gpui's `deferred` does. gpui cannot defer a draw inside a deferred one, so what is raised inside something raised is laid out and drawn last within it: a list opened in a dialog lies over the dialog.
pub fn raise(child: impl IntoElement) -> Raised {
    let layer = Layer {
        child: child.into_any_element(),
    };
    Raised {
        child: Some(layer.into_any_element()),
        priority: 0,
        nested: false,
    }
}

/// An element drawn over the page; see [`raise`].
pub struct Raised {
    child: Option<AnyElement>,
    priority: usize,
    nested: bool,
}

impl Raised {
    /// A higher priority draws later, over the lower.
    pub fn with_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

impl IntoElement for Raised {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Raised {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        self.nested = nested();
        if self.nested {
            let style = Style {
                position: Position::Absolute,
                ..Style::default()
            };
            return (window.request_layout(style, [], cx), ());
        }
        let child = self.child.as_mut().expect("a raised element lays out once");
        (child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) {
        let child = self.child.take().expect("a raised element prepaints once");
        if !self.nested {
            window.defer_draw(child, window.element_offset(), self.priority);
            return;
        }
        LAYERS.with_borrow_mut(|layers| {
            let queue = layers
                .last_mut()
                .expect("a nested element prepaints in its layer");
            queue.push((child, bounds.origin, self.priority));
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
}

/// What a raised element draws: its content, then what was raised inside it, each laid out on its own and the higher priority last.
struct Layer {
    child: AnyElement,
}

impl IntoElement for Layer {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Runs `f` with a fresh queue on the stack, and hands back what was raised meanwhile.
fn collect(f: impl FnOnce()) -> Queue {
    LAYERS.with_borrow_mut(|layers| layers.push(Vec::new()));
    f();
    LAYERS
        .with_borrow_mut(|layers| layers.pop())
        .expect("a layer pops what it pushed")
}

impl Element for Layer {
    type RequestLayoutState = ();
    type PrepaintState = Vec<AnyElement>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut layout = None;
        collect(|| layout = Some(self.child.request_layout(window, cx)));
        (layout.expect("the child laid out"), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<AnyElement> {
        let mut raised = collect(|| {
            self.child.prepaint(window, cx);
        });
        raised.sort_by_key(|(_, _, priority)| *priority);
        let viewport = window.viewport_size();
        let space = size(
            AvailableSpace::Definite(viewport.width),
            AvailableSpace::Definite(viewport.height),
        );
        raised
            .into_iter()
            .enumerate()
            .map(|(ix, (mut element, at, _))| {
                window.with_element_namespace(("raised", ix), |window| {
                    element.prepaint_as_root(at, space, window, cx)
                });
                element
            })
            .collect()
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        raised: &mut Vec<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        for (ix, element) in raised.iter_mut().enumerate() {
            window.with_element_namespace(("raised", ix), |window| element.paint(window, cx));
        }
    }
}
