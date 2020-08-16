## newrelic-unofficial-rust

It's an unofficial port of the [New Relic Go agent](https://github.com/newrelic/go-agent) to Rust.

Unlike the one based on the C sdk, it is completely thread-safe and works alone.

### Status

- [x] Web transactions
- [x] Non-web transactions
- [ ] Tracking threads in transactions
- [ ] Segments
- [ ] Error reporting
- [x] Transaction sampling
- [ ] Apdex

The library reports itself as Go because the New Relic server (of course) doesn't have a support for Rust.

## Usage

Application setup:

```rust
let license = std::env::var("NEW_RELIC_LICENSE_KEY").unwrap();
let app = Application::new("rust-test", &license).unwrap();
```

Transaction:

```rust
// Start a new (non-web) transaction.
// The end of the transaction is automatically recorded on drop.
let txn = app.start_transaction("SomeBackgroundJob");

// Or you can start a web transaction.
let txn = app.start_web_transaction("/upload", http_request);
```

Segment: not yet implemented

## License

I consider it a port of the [New Relic Go agent](https://github.com/newrelic/go-agent), therefore (perhaps) inheriting copyrights from the original source code.

- Copyright 2020 New Relic Corporation. (for the original go-agent)
- Copyright 2020 Masaki Hara.

Licensed under Apache-2.0
