# Twilly

`Twilly` is a workspace containing two separate, but interrelated Rust libraries for interacting with Twilio.

- [`twilly`](#twilly) - A Rust helper library for the Twilio API.
- [`twilly_cli`](#twilly_cli) - CLI application for interacting with Twilio via the terminal built upon `twilly`.

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

[Read more in the library folder.](./twilio_rust/README.md)

## twilly_cli

`twilly_cli` is a command line application for interacting with Twilio via the terminal. It is designed essentially as an _alternative_ [Twilio CLI](https://www.twilio.com/docs/twilio-cli/quickstart), providing easier navigation, less need to remember commands, and helpful utilities approach to various resources.

Under the hood this uses `twilly`, hence the co-location of these libraries.

[Read more in the library folder.](./twilio_cli/README.md)


### Demo

![twilly_cli being used to load an active profile, view stored conversations, and delete a closed conversation on the account with a confirmation prompt](./assets/delete-conversation.gif)
