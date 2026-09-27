use gpui::{AnyElement, Entity, IntoElement, TestAppContext};

use super::{Bench, bench, said, say, settle, tab, tap, write};
use crate::account::{
    BillingHistory, Invitation, InvitationList, Invoice, InvoiceState, Member, PaymentMethodForm,
    Role, Standing, Subscription, SubscriptionCard, TeamMemberTable, UpgradePrompt,
};

fn plan(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let standing = match bench.sent {
        true => Standing::Canceled {
            ends: "1 Oct 2026".into(),
        },
        false => Standing::Active {
            renews: "1 Oct 2026".into(),
        },
    };
    let [cancel, resume] = [(); 2].map(|_| owner.clone());
    SubscriptionCard::new(
        "plan",
        Subscription {
            plan: "Pro".into(),
            price: 20.0,
            currency: "USD".into(),
            yearly: false,
            standing,
        },
    )
    .on_cancel(move |_, cx| say(&cancel, "cancel".into(), cx))
    .on_resume(move |_, cx| say(&resume, "resume".into(), cx))
    .into_any_element()
}

/// Stops: Cancel plan while it runs; Keep plan once canceled.
#[gpui::test]
fn a_plan_cancels_on_the_second_press_and_comes_back(cx: &mut TestAppContext) {
    let (host, cx) = bench(plan, cx);
    tab(1, cx);
    tap("space", cx);
    assert!(said(&host, cx).is_empty(), "the first press only arms");
    tap("space", cx);
    host.update(cx, |bench, cx| {
        bench.sent = true;
        cx.notify();
    });
    settle(cx);
    tab(1, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["cancel", "resume"]);
}

fn card(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    PaymentMethodForm::new("card")
        .on_submit(move |card, _, cx| {
            say(
                &owner,
                format!(
                    "{} {} {}/{} {}",
                    card.brand.name(),
                    card.number,
                    card.month,
                    card.year,
                    card.name
                ),
                cx,
            )
        })
        .into_any_element()
}

/// Stops: number, expiry, code, name, then Save once the card reads.
#[gpui::test]
fn a_card_saves_once_its_number_checks_out(cx: &mut TestAppContext) {
    let (host, cx) = bench(card, cx);
    tab(1, cx);
    write("4242424242424241", cx);
    tab(2, cx);
    write("1240", cx);
    tab(3, cx);
    write("123", cx);
    tab(4, cx);
    write("Ada Lovelace", cx);
    tap("enter", cx);
    assert!(
        said(&host, cx).is_empty(),
        "a number that fails its check keeps Save at rest"
    );
    tab(1, cx);
    tap("backspace", cx);
    write("2", cx);
    tab(4, cx);
    tap("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["Visa 4242424242424242 12/2040 Ada Lovelace"]
    );
}

fn roles() -> Vec<Role> {
    let role = |key: &str, name: &str| Role {
        key: key.to_string().into(),
        name: name.to_string().into(),
        description: "Can do things".into(),
    };
    vec![role("member", "Member"), role("admin", "Admin")]
}

fn invites(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    let sent = Invitation {
        key: "bo".into(),
        email: "bo@example.com".into(),
        role: "admin".into(),
        sent: "Sent 2 days ago".into(),
        expired: false,
    };
    let [invited, again, gone] = [(); 3].map(|_| owner.clone());
    InvitationList::new("invites", [sent], roles())
        .on_invite(move |email, role, _, cx| say(&invited, format!("invite {email} {role}"), cx))
        .on_resend(move |key, _, cx| say(&again, format!("resend {key}"), cx))
        .on_revoke(move |key, _, cx| say(&gone, format!("revoke {key}"), cx))
        .into_any_element()
}

/// Stops: the address, the role, then Resend and Revoke while Invite rests.
#[gpui::test]
fn an_invitation_goes_as_a_member_and_one_out_resends(cx: &mut TestAppContext) {
    let (host, cx) = bench(invites, cx);
    tab(3, cx);
    tap("space", cx);
    tab(1, cx);
    write("cy@example.com", cx);
    tap("enter", cx);
    assert_eq!(
        said(&host, cx),
        ["resend bo", "invite cy@example.com member"]
    );
}

fn team(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    let member = |key: &str, owner: bool| Member {
        key: key.to_string().into(),
        name: key.to_string().into(),
        email: format!("{key}@example.com").into(),
        role: "member".into(),
        seen: "Active today".into(),
        image: None,
        owner,
    };
    let removed = owner.clone();
    TeamMemberTable::new("team", [member("ada", true), member("bo", false)], roles())
        .on_role(move |key, role, _, cx| say(&owner, format!("{key} {role}"), cx))
        .on_remove(move |key, _, cx| say(&removed, format!("remove {key}"), cx))
        .into_any_element()
}

/// Stops: bo's role, then bo's Remove; the owner has neither.
#[gpui::test]
fn a_member_leaves_on_the_second_press_and_the_owner_stays(cx: &mut TestAppContext) {
    let (host, cx) = bench(team, cx);
    tab(2, cx);
    tap("space", cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["remove bo"]);
}

fn wall(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    let later = owner.clone();
    UpgradePrompt::new("wall", "Unlock history")
        .benefit("Every message, kept")
        .on_upgrade(move |_, cx| say(&owner, "upgrade".into(), cx))
        .on_dismiss(move |_, cx| say(&later, "later".into(), cx))
        .into_any_element()
}

/// Stops: Upgrade, then Not now.
#[gpui::test]
fn a_paywall_upgrades_or_waits(cx: &mut TestAppContext) {
    let (host, cx) = bench(wall, cx);
    tab(1, cx);
    tap("space", cx);
    tab(2, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["upgrade", "later"]);
}

fn busy_card(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    PaymentMethodForm::new("card")
        .busy(true)
        .on_submit(move |card, _, cx| say(&owner, card.number.to_string(), cx))
        .into_any_element()
}

/// Stops: number, expiry, code, name; Save rests while the owner checks the card.
#[gpui::test]
fn a_card_waits_while_the_owner_checks_it(cx: &mut TestAppContext) {
    let (host, cx) = bench(busy_card, cx);
    tab(1, cx);
    write("4242424242424242", cx);
    tab(2, cx);
    write("1240", cx);
    tab(3, cx);
    write("123", cx);
    tab(4, cx);
    write("Ada Lovelace", cx);
    tap("enter", cx);
    assert!(said(&host, cx).is_empty());
}

/// Stops: the address, the role; Invite waits for an address that reads.
#[gpui::test]
fn an_invitation_waits_for_an_address(cx: &mut TestAppContext) {
    let (host, cx) = bench(invites, cx);
    tab(1, cx);
    write("cy@example", cx);
    tap("enter", cx);
    assert!(said(&host, cx).is_empty());
}

/// Stops: bo's role first; Enter opens its list, Down reaches Admin and Enter picks it.
#[gpui::test]
fn a_members_role_changes_from_its_row(cx: &mut TestAppContext) {
    let (host, cx) = bench(team, cx);
    tab(1, cx);
    for key in ["enter", "down", "enter"] {
        tap(key, cx);
    }
    assert_eq!(said(&host, cx), ["bo admin"]);
}

fn bills(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    let invoice = Invoice {
        key: "sep".into(),
        date: "1 Sep 2026".into(),
        description: "Pro, monthly".into(),
        amount: 20.0,
        currency: "USD".into(),
        state: InvoiceState::Paid,
    };
    BillingHistory::new("bills", [invoice])
        .on_download(move |key, _, cx| say(&owner, format!("download {key}"), cx))
        .into_any_element()
}

/// Stops: the bill's download.
#[gpui::test]
fn a_bill_downloads_from_its_row(cx: &mut TestAppContext) {
    let (host, cx) = bench(bills, cx);
    tab(1, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["download sep"]);
}

/// Stops: number, expiry, code, name, then Save once the card reads.
#[gpui::test]
fn a_card_waits_for_a_whole_code_and_a_name(cx: &mut TestAppContext) {
    let (host, cx) = bench(card, cx);
    tab(1, cx);
    write("4242424242424242", cx);
    tab(2, cx);
    write("1240", cx);
    tab(3, cx);
    write("12", cx);
    tab(4, cx);
    write("Ada", cx);
    tap("enter", cx);
    assert!(said(&host, cx).is_empty(), "a code of two digits");
    tab(3, cx);
    write("3", cx);
    tab(4, cx);
    tap("cmd-a", cx);
    tap("backspace", cx);
    tap("enter", cx);
    assert!(said(&host, cx).is_empty(), "no name on the card");
    write("Ada", cx);
    tap("enter", cx);
    assert_eq!(said(&host, cx), ["Visa 4242424242424242 12/2040 Ada"]);
}
