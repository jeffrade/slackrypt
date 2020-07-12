# Slackrypt Server

_Disclaimer: This project has not been audited and not recommended for production environments._

<img src="https://github.com/jeffrade/slackrypt/blob/master/slackrypt.jpg" alt="logo" width="128" height="128">

## Prerequisites
 - A [Slack organization](https://slack.com/get-started).
 - Slack [bot keys](https://slack.com/get-started) where you must create a [Classic App](https://api.slack.com/rtm#create_a_classic_slack_app).
 - Have SQLite3 version 3.7.3 or later installed (tested on 3.11.0)
 - You must build from source, so [rustup](https://rustup.rs/).
 - On Linux:
```
$ sudo apt-get install build-essential
$ sudo apt-get install cmake
$ sudo apt-get install sqlite3 libsqlite3-dev
```

## Build
```
$ source .env
$ cargo build
```

## Run
```
$ cargo run
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
## Backlog
#### SERVER
