FROM rustlang/rust:nightly-buster AS builder
WORKDIR /rust/src/github.com/jeffrade/slackrypt/
RUN apt-get update \
  && apt-get -y install build-essential \
  cmake \
  libssl-dev \
  pkg-config 
COPY . .
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app
RUN apt-get update \
  && apt-get -y install ca-certificates \
  libssl-dev \
  sqlite3 \
  libsqlite3-dev
RUN mkdir -p ${APP}
COPY --from=builder /rust/src/github.com/jeffrade/slackrypt/target/release/slackrypt-server ${APP}/slackrypt-server
RUN chmod 655 ${APP}/slackrypt-server
WORKDIR ${APP}
CMD ["./slackrypt-server"]