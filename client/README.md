# Slackrypt Client

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/images/slackrypt.jpg" alt="logo" width="128" height="128">

## Prerequisites
 - Currently, you must build from source, so [rustup](https://rustup.rs/).
 - Have `openssl` installed (verify with `command -v openssl`).
 - A running [slackrypt-server](https://github.com/jeffrade/slackrypt/tree/master/server) hosted over https (see [server/README.md](https://github.com/jeffrade/slackrypt/blob/master/server/README.md) for instructions).
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
