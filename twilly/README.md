
## twilly

`twilly` is a helper library bringing access to Twilio's API's via Rust. The library supports a client-based approach, instantiating a twilio client with credentials before sending & receiving requests.

```rust
let config =  TwilioConfig {
  account_sid: "AC....",
  auth_token: "auth_tok",
};
let twilio = twilly::Client::new(config);

...

let account = twilio.create_account(Some(&friendly_name))
```

Coverage is limited and the crate has been built alongside [`twilly_cli`](https://crates.io/crates/twilly_cli) but can be used independently:

```bash
cargo add twilly
```

or add `twilly` to your `cargo.toml` file:

```toml
[...]

[dependencies]
twilly = "0.1.0"

[...]
```
