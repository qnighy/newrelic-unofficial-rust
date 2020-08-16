// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use newrelic_unofficial::Application;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let license = std::env::var("NEW_RELIC_LICENSE_KEY").unwrap();
    let app = Application::new("rust-test", &license).unwrap();
    for _ in 0..120 {
        let txn = app.start_transaction("test");
        sleep(Duration::from_millis(500));
        drop(txn);
        sleep(Duration::from_millis(500));
    }
}
