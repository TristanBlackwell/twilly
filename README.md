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

`twilly_cli` is a command line application for interacting with Twilio via the terminal. It is decisioned essentially as an _improved_ [Twilio CLI](https://www.twilio.com/docs/twilio-cli/quickstart), providing easier navigation around commands, less need to remember commands, and helpful utilities approach to various resources.

Under the hood this uses `twilly`, hence the co-location of these libraries.

### Demo

![twilly_cli being used to load an active profile, view stored conversations, and delete a closed conversation on the account with a confirmation prompt](./assets/delete-conversation.gif)

## Installation

### Using Rust

Assuming you have the Rust toolchain, you can run the application directly:

```sh
cargo r
```

### Linux

```sh
# Download the packaged binary
wget -q https://github.com/TristanBlackwell/twilly/releases/download/v0.1.1/twilly_cli-v0.1.1-x86_64-unknown-linux-musl.tar.gz

# Unpack
tar xf twilly_cli-v0.1.1-x86_64-unknown-linux-musl.tar.gz

# Move twilly_cli to system path
mv twilly_cli-v0.1.1-x86_64-unknown-linux-musl/twilly_cli /usr/local/bin

# Cleanup
rm -r twilly_cli-v0.1.1-x86_64-unknown-linux-musl twilly_cli-v0.1.1-x86_64-unknown-linux-musl.tar.gz

# Run it
twilly_cli
```

### MacOS

```sh
# Download the binary
wget -q https://github.com/TristanBlackwell/twilly/releases/download/v0.1.1/twilly_cli-v0.1.1-x86_64-apple-darwin.tar.gz

tar xf twilly_cli-v0.1.1-x86_64-apple-darwin.tar.gz

mv twilly_cli-v0.1.1-x86_64-apple-darwin/twilly_cli /usr/local/bin

rm -r twilly_cli-v0.1.1-x86_64-apple-darwin twilly_cli-v0.1.1-x86_64-apple-darwin.tar.gz

twilly_cli
```
