#!/bin/bash

set -e

BLUE='\033[94m'
GREEN='\033[32;1m'
RED='\033[91;1m'
RESET='\033[0m'

print_info() {
    printf "$BLUE$1$RESET\n"
}

print_success() {
    printf "$GREEN$1$RESET\n"
    sleep 1
}

print_error() {
    printf "$RED$1$RESET\n"
    sleep 1
}

if ! [[ $(command -v rustc) ]]; then
    print_error "You must have Rust installed. See https://rustup.rs/"
    exit 1;
fi

if ! [[ $(command -v sqlite3) ]]; then
    print_error "You must have SQLite3 installed!"
    exit 1;
fi

print_info "Staring release..."
print_info "Building..."
cargo build --release
print_info "Exporting environment vars..."
export RUST_LOG=INFO
print_info "Staring slackrypt-server..."
nohup ../target/release/slackrypt-server &

print_success "Deployment complete!"
