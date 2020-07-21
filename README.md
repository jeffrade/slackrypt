# Slackrypt

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/images/slackrypt.jpg" alt="logo" width="128" height="128">

## Prerequisites
 - You must build from source, so [rustup](https://rustup.rs/).
 - Have `openssl` installed (verify with `command -v openssl`).
 - A running [slackrypt-server](https://github.com/jeffrade/slackrypt/tree/master/server) instance to connect to (currently only supports localhost).
 - On Linux:
```
$ sudo apt-get install build-essential
$ sudo apt-get install cmake
$ sudo apt-get install libssl-dev
$ sudo apt-get install pkg-config
$ sudo apt-get install libx11-dev libxext-dev libxft-dev libxinerama-dev libxcursor-dev libxrender-dev libxfixes-dev libgl1-mesa-dev libglu1-mesa-dev
```

## Build
```
$ cargo build
```

## Run
```
$ cargo run
```

## In Action

#### Encrypting...

<img src="https://github.com/jeffrade/slackrypt/blob/master/images/slackrypt-encrypt.gif" alt="encrypt-gif" width="1000">

#### Decrypting...

<img src="https://github.com/jeffrade/slackrypt/blob/master/images/slackrypt-decrypt.gif" alt="decrypt-gif" width="1000">

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
 - Pull Requests and Issues are welcome!
 - Uses `openssl` for key generation.
 - Uses https://github.com/RustCrypto/RSA for parsing/loading keys, encrypting and decrypting symmetric keys.
 - Uses [aes-soft](https://github.com/RustCrypto/block-ciphers/#supported-algorithms) for symmetric encryption of plaintext.

## Backlog
 - Implement "New Public Key" in GUI menu.
#### CRC
 - radix-64 CRC (Cyclic_redundancy_check), in C https://tools.ietf.org/html/rfc4880#section-6.1
