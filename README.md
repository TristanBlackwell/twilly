# Twilio Rust

`Twilio Rust` is a workspace containing two separate, but interrelated Rust libraries for interacting with Twilio.

- [`twilio_rust`](#twilio-rust) - A Rust helper library for the Twilio API.
- [`twilio_cli`](#twilio_cli) - CLI application for interacting with Twilio via the terminal. 

## twilio_rust

`twilio_rust` is a helper library bringing access to Twilio's API's via Rust. The library supports a client-based approach, instantiating a twilio client with credentials before sending & receiving requests.

```rust
let config =  TwilioConfig {
  account_sid: "AC....",
  auth_token: "auth_tok",
};
let twilio = twilio_rust::Client::new(config);

...

let account = twilio.create_account(Some(&friendly_name))
```

[Read more in the library folder.](./twilio_rust/README.md)

## twilio_cli

`twilio_cli` is a command line application for interacting with Twilio via the terminal. It is decisioned essentially as a _better_ [Twilio CLI](https://www.twilio.com/docs/twilio-cli/quickstart), providing easier navigation around commands, less need to remember commands, and helpful utilities approach to various resources.

Under the hood this uses `twilio_rust`, hence the co-location of these libraries. 

### Demo

[Read more in the library folder.](./twilio_cli/README.md)