use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div,
};
use jiff::Timestamp;

use super::login::{Run, field};
use crate::{
    buttons::{Button, ButtonVariant},
    feedback::InlineMessage,
    forms::{Enter, FormField, Input, MaskedInput},
    primitives::{Icon, IconName, Severity},
    theme::{ActiveTheme, IconSize},
    typography::format::system_zone,
};

/// A card network, told by a number's first digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Discover,
    Other,
}

impl CardBrand {
    pub fn of(digits: &str) -> Self {
        let lead = |n: usize| digits.get(..n).and_then(|lead| lead.parse::<u32>().ok());
        if digits.starts_with('4') {
            CardBrand::Visa
        } else if lead(2).is_some_and(|lead| (51..=55).contains(&lead))
            || lead(4).is_some_and(|lead| (2221..=2720).contains(&lead))
        {
            CardBrand::Mastercard
        } else if lead(2).is_some_and(|lead| lead == 34 || lead == 37) {
            CardBrand::Amex
        } else if digits.starts_with("6011") || digits.starts_with("65") {
            CardBrand::Discover
        } else {
            CardBrand::Other
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CardBrand::Visa => "Visa",
            CardBrand::Mastercard => "Mastercard",
            CardBrand::Amex => "American Express",
            CardBrand::Discover => "Discover",
            CardBrand::Other => "Card",
        }
    }

    /// The number's pattern and its digits, and the digits of its security code.
    fn shape(self) -> (&'static str, usize, usize) {
        match self {
            CardBrand::Amex => ("9999 999999 99999", 15, 4),
            _ => ("9999 9999 9999 9999", 16, 3),
        }
    }
}

/// Whether `digits` pass the Luhn check that card numbers carry.
pub fn luhn(digits: &str) -> bool {
    let mut sum = 0;
    for (ix, ch) in digits.chars().rev().enumerate() {
        let Some(mut digit) = ch.to_digit(10) else {
            return false;
        };
        if ix % 2 == 1 {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }
        sum += digit;
    }
    !digits.is_empty() && sum % 10 == 0
}

/// The month and year of an expiry written "MM/YY", when it reads and has not passed `now`, a year and month.
pub fn expiry(text: &str, now: (i32, i8)) -> Option<(i8, i32)> {
    let (month, year) = text.split_once('/')?;
    let (month, year) = (
        month.trim().parse::<i8>().ok()?,
        year.trim().parse::<i32>().ok()?,
    );
    let year = 2000 + year;
    ((1..=12).contains(&month) && (year, month) >= now).then_some((month, year))
}

/// A card as its form hands it over.
#[derive(Clone, Debug, PartialEq)]
pub struct CardDetails {
    pub brand: CardBrand,
    pub number: SharedString,
    pub month: i8,
    pub year: i32,
    pub cvc: SharedString,
    pub name: SharedString,
}

type OnCard = Rc<dyn Fn(&CardDetails, &mut Window, &mut App)>;

fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

/// A card to pay with: its number, spaced as its network writes it and named once told, the month and year it runs out, its security code and the name on it. Save waits for a number that checks out, a month ahead, a full code and a name.
#[derive(IntoElement)]
pub struct PaymentMethodForm {
    id: ElementId,
    error: Option<SharedString>,
    busy: bool,
    on_submit: Option<OnCard>,
}

impl PaymentMethodForm {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            error: None,
            busy: false,
            on_submit: None,
        }
    }

    /// Why the card was refused, shown above Save.
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// While the owner checks the card.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn on_submit(
        mut self,
        handler: impl Fn(&CardDetails, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for PaymentMethodForm {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let on_submit = self
            .on_submit
            .unwrap_or_else(|| panic!("payment method form {id:?} has no on_submit"));
        let number = field(&id, "number", "", false, window, cx);
        let until = field(&id, "expiry", "", false, window, cx);
        let code = field(&id, "cvc", "", false, window, cx);
        let name = field(&id, "name", "As printed on the card", false, window, cx);
        let typed = digits(number.read(cx).text());
        let brand = CardBrand::of(&typed);
        let (mask, length, code_length) = brand.shape();
        let complete = typed.len() == length;
        let checks = complete && luhn(&typed);
        let now = Timestamp::now().to_zoned(system_zone("payment method form"));
        let expires = expiry(until.read(cx).text(), (now.year() as i32, now.month()));
        let cvc = digits(code.read(cx).text());
        let holder: SharedString = name.read(cx).text().trim().to_string().into();
        let card = expires
            .filter(|_| checks && cvc.len() == code_length && !holder.is_empty())
            .map(|(month, year)| CardDetails {
                brand,
                number: typed.clone().into(),
                month,
                year,
                cvc: cvc.clone().into(),
                name: holder,
            });
        let ready = card.is_some() && !self.busy;
        let submit: Run = Rc::new(move |window, cx| {
            let card = card.as_ref().expect("Save rests until the card reads");
            log::info!("payment method form: save {}", card.brand.name());
            on_submit(card, window, cx);
        });
        let enter = submit.clone();
        let theme = cx.theme();
        let mut number_field = FormField::new((id.clone(), "number-field"), "Card number");
        if complete && !checks {
            number_field = number_field.error("This number does not check out.");
        } else if brand != CardBrand::Other {
            number_field = number_field.description(brand.name());
        }
        let late = until.read(cx).text().len() == "MM/YY".len() && expires.is_none();
        let mut until_field = FormField::new((id.clone(), "expiry-field"), "Expires");
        if late {
            until_field = until_field.error("This date has passed.");
        }
        div()
            .flex()
            .flex_col()
            .gap_4()
            .capture_action(move |_: &Enter, window, cx| {
                if ready {
                    cx.stop_propagation();
                    enter(window, cx);
                }
            })
            .child(
                number_field.child(
                    MaskedInput::new((id.clone(), "number"), &number, mask).prefix(
                        Icon::new(IconName::CreditCard)
                            .size(IconSize::Sm)
                            .color(theme.colors.fg_subtle),
                    ),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .child(div().flex_1().min_w(theme.label_width() * 0.6).child(
                        until_field.child(MaskedInput::new(
                            (id.clone(), "expiry"),
                            &until,
                            "99/99",
                        )),
                    ))
                    .child(div().flex_1().min_w(theme.label_width() * 0.6).child(
                        FormField::new((id.clone(), "cvc-field"), "Security code").child(
                            MaskedInput::new(
                                (id.clone(), "cvc"),
                                &code,
                                if code_length == 4 { "9999" } else { "999" },
                            ),
                        ),
                    )),
            )
            .child(
                FormField::new((id.clone(), "name-field"), "Name on card").child(Input::new(&name)),
            )
            .children(
                self.error
                    .map(|error| InlineMessage::new(Severity::Danger, error)),
            )
            .child(
                Button::new((id.clone(), "save"), "Save card")
                    .variant(ButtonVariant::Primary)
                    .full_width()
                    .loading(self.busy)
                    .disabled(!ready)
                    .on_click(move |_, window, cx| submit(window, cx)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{CardBrand, expiry, luhn};

    #[test]
    fn a_number_tells_its_network_and_checks_out() {
        assert_eq!(CardBrand::of("4242"), CardBrand::Visa);
        assert_eq!(CardBrand::of("5555"), CardBrand::Mastercard);
        assert_eq!(CardBrand::of("2221"), CardBrand::Mastercard);
        assert_eq!(CardBrand::of("3782"), CardBrand::Amex);
        assert_eq!(CardBrand::of("6011"), CardBrand::Discover);
        assert_eq!(CardBrand::of("9"), CardBrand::Other);
        assert!(luhn("4242424242424242"));
        assert!(luhn("378282246310005"));
        assert!(!luhn("4242424242424241"));
        assert!(!luhn(""));
    }

    #[test]
    fn an_expiry_reads_and_has_not_passed() {
        let now = (2026, 9);
        assert_eq!(
            expiry("09/26", now),
            Some((9, 2026)),
            "this month still counts"
        );
        assert_eq!(expiry("12/31", now), Some((12, 2031)));
        assert_eq!(expiry("08/26", now), None, "last month has passed");
        assert_eq!(expiry("13/30", now), None, "there is no thirteenth month");
        assert_eq!(expiry("1230", now), None);
    }
}
