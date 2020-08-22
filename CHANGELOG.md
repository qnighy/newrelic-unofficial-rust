## Unreleased

- Rename `Daemon` as `ApplicationGuard` and make it deref to `Application`
- Split `Transaction` into `Transaction` and `TransactionGuard`
- Add `WebRequest` for `start_web_transaction` argument

## 0.1.3

- Get rid of `eprintln` logs

## 0.1.2

- Limit number of txn traces
- Internal refactorings

## 0.1.1

- Basic web transaction support
- Transaction samples (no traffic control yet)
- Bunch of internal protocol improvements
- Internal refactorings

## 0.1.0

Initial release with the basic non-web transaction support