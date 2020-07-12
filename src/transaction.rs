use std::sync::Weak;
use std::time::Instant;

use crate::ApplicationInner;

#[derive(Debug)]
pub struct Transaction {
    app: Weak<ApplicationInner>,
    start: Instant,
}
