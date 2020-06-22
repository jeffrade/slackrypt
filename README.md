# Slackrypt

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/slackrypt.jpg" alt="logo" width="128" height="128">

## Prerequisites
 - You must build from source, so [rustup](https://rustup.rs/).
 - Have `openssl` installed (verify with `command -v openssl`).
 - A running [slackrypt-server](https://github.com/jeffrade/slackrypt/tree/master/server) instance to connect to (currently only supports localhost).

## Build
```
$ source .env
$ cargo build
```

## Run
```
$ cargo run "Your plaintext message"
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
 - Uses `openssl` for key generation.
 - Uses https://github.com/RustCrypto/RSA for parsing/loading keys, encrypting and decrypting plaintext.
 - Uses [aes-soft](https://github.com/RustCrypto/block-ciphers/#supported-algorithms) for symmetric encryption.

## Backlog
#### CRC
 - radix-64 CRC (Cyclic_redundancy_check), in C https://tools.ietf.org/html/rfc4880#section-6.1
