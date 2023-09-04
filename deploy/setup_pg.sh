#!/bin/bash

set -euxo pipefail

cd /setup

# wait for cloud-init to finish
/usr/bin/cloud-init status --wait

# setup postgres
#sudo -u postgres psql -c "ALTER ROLE postgres PASSWORD '${POSTGRES_PASSWORD}'"

# configure impulse postgres
#if [[ "${POSTGRES_PURPOSE}" == "impulse" ]]; then
#  sudo -u postgres psql -c "CREATE DATABASE impulse"
#  sudo cp image_files/postgresql.conf /etc/postgresql/14/main/postgresql.conf
#  sudo systemctl restart postgresql
#  # database migration
#  # diesel requirements
#  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh
#  chmod +x install_rustup.sh
#  ./install_rustup.sh -y
#  source "$HOME/.cargo/env"
#  cargo install diesel_cli
#  cp image_files/.env .
#  diesel migration run
#fi

# configure managed postgres
#if [[ "${POSTGRES_PURPOSE}" == "managed" ]]; then
#  pushd image_files
#  ../release/setup_database --host localhost
#  popd
#fi

# modify firewall
#ufw allow 5432
