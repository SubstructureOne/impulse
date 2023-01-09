#!/bin/bash

set -ux

echo "Starting Postgres container"
CONTAINER_ID=`cargo run --bin setup_database`
echo "Running tests"
RUST_LOG=trace RUST_BACKTRACE=1 cargo test
echo "Stopping and removing Postgres container"
docker stop "${CONTAINER_ID}"
docker rm "${CONTAINER_ID}"
echo "Done."
