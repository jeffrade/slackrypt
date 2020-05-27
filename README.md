# Slackrypt

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/slackrypt.jpg" alt="logo" width="128" height="128">

## Prerequisites
 - A [Slack organization](https://slack.com/get-started).
 - Slack [bot keys](https://slack.com/get-started).
 - You must build from source, so [rustup](https://rustup.rs/). 
 - `git clone` https://github.com/RustCrypto/RSA into same parent dir as this project.

## Build
```
$ source .env
$ cargo build
```

## Run
```
$ cargo run "Your plaintext message"
```

## Deploy
```
$ bash deploy.sh
```

## Logging
 - Defaults to `ERROR` when `RUST_LOG` is not set.
 - You can `export` per environment. E.g.:
```
# for local development
$ export RUST_LOG=DEBUG
# but in a production environment:
$ export RUST_LOG=WARN
```

## Information
 - Uses `openssl` for key generation
 - Uses https://github.com/RustCrypto/RSA for parsing/loading keys, encrypting and decrypting plaintext.

## Backlog
#### Slack
 - https://api.slack.com/events-api
 - https://api.slack.com/rtm
 - https://api.slack.com/apps/A011BQES6MC/general?
 - https://github.com/lins05/slackbot (for reference)
 - https://github.com/slack-rs/slack-rs
 - https://github.com/slack-rs/slack-rs/blob/a6c2fbd5a17c2831a17560b6ebcdce60ce595f18/examples/slack_example.rs

#### AES
 - https://github.com/RustCrypto/block-ciphers#usage
 - https://docs.rs/aes-soft/0.3.3/aes_soft/ 
