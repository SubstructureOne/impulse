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

echo "Starting Postgres container"
CONTAINER_ID=$(cargo run --bin setup_database)
echo "Running tests"
set +e
TESTING_BASE_URL="postgres://postgres:${DOCKER_POSTGRES_PASSWORD}@localhost:${DOCKER_POSTGRES_PORT}" \
    TESTING_DB_PORT=${DOCKER_POSTGRES_PORT} \
    RUST_LOG=trace \
    RUST_BACKTRACE=1 \
    cargo test
set -e
echo "Stopping and removing Postgres container"
docker stop "${CONTAINER_ID}"
docker rm "${CONTAINER_ID}"
echo "Done."
