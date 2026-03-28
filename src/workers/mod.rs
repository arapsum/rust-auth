mod mail_queue;

pub use self::mail_queue::{MailJob, MailQueue, handle_forgot_password, handle_welcome};
