mod boxes;
mod list;
mod quoted;
mod reader;
mod thread;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use boxes::{Mailbox, MailboxList};
pub use list::{Mail, MailItem, MailList};
pub use quoted::QuotedText;
pub use reader::{Contact, MailReader, Message};
pub use thread::MailThreadView;
