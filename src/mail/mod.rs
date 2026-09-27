mod boxes;
mod list;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use boxes::{Mailbox, MailboxList};
pub use list::{Mail, MailItem, MailList};
