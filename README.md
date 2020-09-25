# Slackrypt

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/images/slackrypt.jpg" alt="logo" width="128" height="128">

### Client

See [client/README.md](https://github.com/jeffrade/slackrypt/blob/master/client)

### Server

See [server/README.md](https://github.com/jeffrade/slackrypt/blob/master/server)

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
 - Add Mac OSX buld instructions.
 - Implement "New Public Key" in GUI menu.
 - radix-64 CRC (Cyclic_redundancy_check), in C https://tools.ietf.org/html/rfc4880#section-6.1
