use std::rc::Rc;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px,
};
use jiff::Timestamp;

use crate::finance::{CandlestickChart, ChartSync, candles::Candle, stage::Visible};
use crate::theme::Theme;

fn candles(count: usize) -> Rc<Vec<Candle>> {
    Rc::new(
        (0..count)
            .map(|ix| {
                let time = Timestamp::from_second(ix as i64 * 86_400).expect("a day");
                Candle::new(time, (10.0, 11.0, 9.0, 10.0 + ix as f64 * 0.01), 100.0)
            })
            .collect(),
    )
}

struct Pair {
    first: Rc<Vec<Candle>>,
    second: Rc<Vec<Candle>>,
    sync: Entity<ChartSync>,
}

impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(800.))
            .child(CandlestickChart::new("first", self.first.clone()).sync(&self.sync))
            .child(CandlestickChart::new("second", self.second.clone()).sync(&self.sync))
    }
}

/// Two synced charts of 40 candles, their shared window held on the newest 20.
fn held(cx: &mut TestAppContext) -> (Entity<Pair>, Entity<ChartSync>, &mut VisualTestContext) {
    cx.update(Theme::init);
    let sync = cx.new(|_| ChartSync::default());
    let shared = sync.clone();
    let (view, cx) = cx.add_window_view(|_, _| Pair {
        first: candles(40),
        second: candles(40),
        sync: shared,
    });
    cx.run_until_parked();
    sync.update(cx, |sync, cx| {
        sync.visible = Some(Visible {
            start: 20.0,
            count: 20.0,
        });
        cx.notify();
    });
    view.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    (view, sync, cx)
}

fn start(sync: &Entity<ChartSync>, cx: &mut VisualTestContext) -> f64 {
    sync.read_with(cx, |sync, _| sync.visible.expect("held").start)
}

#[gpui::test]
fn synced_charts_move_their_shared_window_once_per_append(cx: &mut TestAppContext) {
    let (view, sync, cx) = held(cx);
    view.update(cx, |pair, cx| {
        pair.first = candles(45);
        pair.second = candles(45);
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        start(&sync, cx),
        25.0,
        "five new candles move the shared window five, not ten"
    );
}

#[gpui::test]
fn a_lagging_chart_leaves_the_shared_window_alone(cx: &mut TestAppContext) {
    let (view, sync, cx) = held(cx);
    view.update(cx, |pair, cx| {
        pair.second = candles(45);
        cx.notify();
    });
    cx.run_until_parked();
    for _ in 0..2 {
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    assert_eq!(
        (start(&sync, cx), sync.read_with(cx, |sync, _| sync.seen)),
        (25.0, 45),
        "redraws while the first chart lags move nothing"
    );
    view.update(cx, |pair, cx| {
        pair.first = candles(45);
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        start(&sync, cx),
        25.0,
        "the lagging chart's append is already shown"
    );
}
