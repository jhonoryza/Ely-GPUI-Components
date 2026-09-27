mod boxes;
mod compose;
mod list;
mod quoted;
mod reader;
mod send;
mod signature;
mod thread;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use boxes::{Mailbox, MailboxList};
pub use compose::{Draft, MailComposer};
pub use list::{Mail, MailItem, MailList};
pub use quoted::QuotedText;
pub use reader::{Contact, MailReader, Message};
pub use send::ScheduleSend;
pub use signature::SignatureEditor;
pub use thread::MailThreadView;
