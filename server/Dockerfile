FROM rustlang/rust:nightly-buster

RUN apt-get update \
  && apt-get -y install build-essential \
  cmake \
  libssl-dev \
  pkg-config \
  sqlite3 \
  libsqlite3-dev

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/slackrypt-server"]