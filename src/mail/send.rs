use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*};
use jiff::{Timestamp, ToSpan, civil::DateTime, tz::TimeZone};

use crate::{
    buttons::{Button, ButtonVariant},
    forms::{DateTimePicker, Run},
    menus::{Menu, MenuItem, SplitButton},
    overlays::Dialog,
    primitives::IconName,
    typography::format,
};

pub(super) type OnTime = Rc<dyn Fn(Timestamp, &mut Window, &mut App)>;

/// Times to send later from `now` in `zone`: tomorrow at 8:00 and at 13:00, and the next Monday at 8:00, each named with its day and hour.
fn presets(now: Timestamp, zone: &TimeZone) -> [(String, Timestamp); 3] {
    let today = now.to_zoned(zone.clone()).date();
    let tomorrow = today.tomorrow().expect("a day follows today");
    let ahead = 7 - i64::from(today.weekday().to_monday_zero_offset());
    let monday = today
        .checked_add(ahead.days())
        .expect("a Monday lies ahead");
    let at = |day: jiff::civil::Date, hour: i8| {
        day.at(hour, 0, 0, 0)
            .to_zoned(zone.clone())
            .expect("the zone holds the hour")
            .timestamp()
    };
    let named = |name: &str, when: Timestamp| {
        let hour = format::datetime(when, zone, "%a %H:%M").expect("a fixed pattern formats");
        (format!("{name} · {hour}"), when)
    };
    [
        named("Tomorrow morning", at(tomorrow, 8)),
        named("Tomorrow afternoon", at(tomorrow, 13)),
        named("Monday morning", at(monday, 8)),
    ]
}

/// Whether the dialog for a time of one's own shows, and the time picked in it.
#[derive(Default)]
struct Picking {
    open: bool,
    chosen: Option<DateTime>,
}

/// Send, and beside it the times to send later: tomorrow morning, tomorrow afternoon and Monday morning in the zone given, or a time picked in a dialog. Each only with its handler; disabled, only Send shows, dimmed.
#[derive(IntoElement)]
pub struct ScheduleSend {
    id: ElementId,
    zone: Option<TimeZone>,
    disabled: bool,
    on_send: Option<Run>,
    on_schedule: Option<OnTime>,
}

impl ScheduleSend {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            zone: None,
            disabled: false,
            on_send: None,
            on_schedule: None,
        }
    }

    /// The zone its times read in; the system's otherwise.
    pub fn zone(mut self, zone: TimeZone) -> Self {
        self.zone = Some(zone);
        self
    }

    /// Nothing to send yet.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_send(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(handler));
        self
    }

    /// Gets the moment to send at.
    pub fn on_schedule(
        mut self,
        handler: impl Fn(Timestamp, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_schedule = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ScheduleSend {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let zone = self
            .zone
            .unwrap_or_else(|| format::system_zone("schedule send"));
        let picking =
            window.use_keyed_state((self.id.clone(), "picking"), cx, |_, _| Picking::default());
        let menu = self
            .on_schedule
            .clone()
            .filter(|_| !self.disabled)
            .map(|schedule| {
                let presets = presets(Timestamp::now(), &zone).into_iter().fold(
                    Menu::new(),
                    |menu, (name, at)| {
                        let schedule = schedule.clone();
                        menu.item(MenuItem::new(name).on_click(move |window, cx| {
                            log::info!("schedule send: at {at}");
                            schedule(at, window, cx)
                        }))
                    },
                );
                let opening = picking.clone();
                presets.separator().item(
                    MenuItem::new("Pick a time…")
                        .icon(IconName::Calendar)
                        .on_click(move |_, cx| {
                            log::info!("schedule send: pick a time");
                            opening.update(cx, |picking, cx| {
                                picking.open = true;
                                cx.notify();
                            })
                        }),
                )
            });
        let send = self.on_send.clone();
        let button = match menu {
            Some(menu) => SplitButton::new((self.id.clone(), "send"), "Send", menu)
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| {
                    log::info!("schedule send: send");
                    if let Some(send) = &send {
                        send(window, cx);
                    }
                })
                .into_any_element(),
            None => Button::new((self.id.clone(), "send"), "Send")
                .variant(ButtonVariant::Primary)
                .disabled(self.disabled)
                .on_click(move |_, window, cx| {
                    log::info!("schedule send: send");
                    if let Some(send) = &send {
                        send(window, cx);
                    }
                })
                .into_any_element(),
        };
        let now = picking.read(cx);
        let dialog = (now.open).then(|| {
            let chosen = now.chosen;
            let (closing, choosing, id) = (picking.clone(), picking.clone(), self.id.clone());
            let schedule = self.on_schedule.clone();
            Dialog::new(
                (self.id.clone(), "dialog"),
                "Schedule send",
                move |_, cx| {
                    closing.update(cx, |picking, cx| {
                        *picking = Picking::default();
                        cx.notify();
                    })
                },
            )
            .detail("Pick the day and the hour it goes.")
            .child(
                DateTimePicker::new((id.clone(), "when"), chosen).on_change(move |at, _, cx| {
                    choosing.update(cx, |picking, cx| {
                        picking.chosen = Some(at);
                        cx.notify();
                    })
                }),
            )
            .action(move |close| {
                Button::new((id.clone(), "cancel"), "Cancel")
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, window, cx| close(window, cx))
            })
            .action({
                let (id, zone) = (self.id.clone(), zone.clone());
                move |close| {
                    Button::new((id, "schedule"), "Schedule")
                        .variant(ButtonVariant::Primary)
                        .disabled(chosen.is_none())
                        .on_click(move |_, window, cx| {
                            let at = chosen
                                .expect("Schedule waits for a time")
                                .to_zoned(zone.clone())
                                .expect("the zone holds the time")
                                .timestamp();
                            log::info!("schedule send: at {at}");
                            close(window, cx);
                            if let Some(schedule) = &schedule {
                                schedule(at, window, cx);
                            }
                        })
                }
            })
        });
        div().flex_none().child(button).children(dialog)
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn at(day: i8, hour: i8) -> Timestamp {
        date(2026, 9, day)
            .at(hour, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .expect("a UTC time")
            .timestamp()
    }

    #[test]
    fn presets_fall_tomorrow_and_on_the_next_monday() {
        let [(morning, a), (afternoon, b), (monday, c)] = presets(at(26, 12), &TimeZone::UTC);
        assert_eq!(
            (morning.as_str(), a),
            ("Tomorrow morning · Sun 08:00", at(27, 8))
        );
        assert_eq!(
            (afternoon.as_str(), b),
            ("Tomorrow afternoon · Sun 13:00", at(27, 13))
        );
        assert_eq!(
            (monday.as_str(), c),
            ("Monday morning · Mon 08:00", at(28, 8))
        );
        let [_, _, (_, next)] = presets(at(28, 9), &TimeZone::UTC);
        assert_eq!(
            next,
            date(2026, 10, 5)
                .at(8, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .expect("a UTC time")
                .timestamp(),
            "on a Monday, the next one"
        );
    }
}
