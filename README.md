## newrelic-unofficial-rust

It's an unofficial port of the [New Relic Go agent](https://github.com/newrelic/go-agent) to Rust.

Unlike the one based on the C sdk, it is completely thread-safe and works alone.

### status

A very basic transaction works. Error handling is mostly lacking.

The library reports itself as Go because the New Relic server (of course) doesn't have a support for Rust.

## License

I consider it a port of the [New Relic Go agent](https://github.com/newrelic/go-agent), therefore (perhaps) inheriting copyrights from the original source code.

- Copyright 2020 New Relic Corporation. (for the original go-agent)
- Copyright 2020 Masaki Hara.

Licensed under Apache-2.0
