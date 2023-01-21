#!/bin/bash

set -eux

# load .env
set -o allexport
source .env
set +o allexport

# compile everything first so we don't waste time setting up the docker
# container if there's a syntax error
cargo build
cargo test --no-run

echo "Starting Impulse Postgres container"
IMPULSE_CONTAINER_ID=$(cargo run --bin create_container -- --name impulse_test --port 6432 --password pw)
echo "Starting Managed Postgres container"
MANAGED_CONTAINER_ID=$(cargo run --bin create_container -- --name impulse_managed_test --port 7432 --password pw)
echo "Configuring Managed Postgres database"
cargo run --bin setup_database -- --port 7432 --password pw --host localhost --username postgres
echo "Running tests"
set +e
TESTING_DB_HOST=localhost \
    TESTING_DB_PORT=6432 \
    TESTING_DB_USER=postgres \
    TESTING_DB_PASSWORD=pw \
    MANAGED_DB_HOST=localhost \
    MANAGED_DB_PORT=7432 \
    MANAGED_DB_USER=postgres \
    MANAGED_DB_PASSWORD=pw \
    RUST_LOG=trace \
    RUST_BACKTRACE=1 \
    cargo test
set -e
echo "Stopping and removing Postgres containers"
docker stop "${IMPULSE_CONTAINER_ID}"
docker rm "${IMPULSE_CONTAINER_ID}"
docker stop "${MANAGED_CONTAINER_ID}"
docker rm "${MANAGED_CONTAINER_ID}"
echo "Done."
