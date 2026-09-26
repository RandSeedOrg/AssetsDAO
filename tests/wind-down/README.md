# Wind-down recovery regression tests

These tests install isolated WASM canisters in PocketIC. They never access a live
ledger or deploy to a shared network. PocketIC **server 11** is required by the
pinned client. Tested with Rust/Cargo 1.94.1. Set `POCKET_IC_BIN` to that local binary.

From this repository root:

```sh
cargo test -p staking
cargo build -p staking --features recovery-tests --target wasm32-unknown-unknown --release --target-dir /tmp/wind-down-fixture-target
cargo build --manifest-path tests/wind-down/mock/Cargo.toml --target wasm32-unknown-unknown --release --target-dir /tmp/wind-down-mock-target
```

Two tests also exercise the actual Motoko pay center. In the parent application
workspace, set `MOC` to its Motoko compiler and run:

```sh
python3 src/pay_center/tests/build_wind_down_fixture.py
```

That script compiles current and HEAD pay-center sources with simulator-only
seeding methods, writes artifacts under `/tmp/wind-down-pay-center`, and checks
stable type compatibility. It does not edit production source. The fixture records
legacy credit without the new full receipt, exercising bounded reconstruction
across an actual upgrade.

Run the suite:

```sh
cargo test --manifest-path tests/wind-down/Cargo.toml -- --test-threads=1
```

Artifact paths can be overridden with `STAKING_FIXTURE_WASM`, `MOCK_WASM`,
`PAY_CENTER_WASM`, and `PREVIOUS_PAY_CENTER_WASM`. The Motoko builder accepts
`WIND_DOWN_TEST_OUTPUT`; set the corresponding WASM variables if changing it.
Cargo dependencies are confined to the isolated test workspace and mock crate;
they do not enter staking production builds. A working Cargo registry/cache and
permission to listen on localhost are required.

## Coverage

- Orphan Release and Dissolve reuse existing blocks with zero sends.
- The pool-3-shaped fixture advances from 16/23 and cursor 146 to 23/23 with exact
  completed amount and virtual/chain balance after processing the seven remaining accounts.
- Released accounts with missing virtual debits recover correctly.
- Index synchronization delays, rejection, wrong ledger pairing, and cross-page
  duplicate transfers never authorize payment.
- Hex addresses are compared as bytes, independent of letter case. An impossible lifecycle
  lower bound beyond the Ledger tip stops before payment.
- At most five account-history pages per Execute; a two-billion-block global tip
  does not increase account-history pages. Saved pagination survives upgrade.
- Trap after a successful Release or Dissolve callback recovers without resending.
- Fixed transfer arguments accept `TxDuplicate`; expired uncertain intent does
  not renew its timestamp or send again, even if an Index incorrectly omits it.
- Archive verification requests exactly one block. The mock rejects all global
  range scans.
- Pay-center response loss credits once. Actual Motoko concurrent replay and
  conflicting requests, plus legacy history over an upgrade, are checked.
- Pause during an in-flight batch remains Paused until an explicit Execute.

## Production boundary

Never enable `recovery-tests` for a deployment or use the fixture WASM. Normal
staking builds exclude its seeding and fault-injection endpoints. Use a separate
target directory for simulator builds, as above. After a production build, extract
Candid and confirm there are no `fixture_` methods.

Historical pool audit is also bounded by the per-call external-call budget. It
may initially show older, already-dissolved accounts: only their known Release
blocks are verified to repair fee records before reconciliation. This is distinct
from the per-account history lookup used for missing receipts.
