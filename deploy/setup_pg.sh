#!/bin/bash

set -euxo pipefail

cd /setup

# wait for cloud-init to finish
/usr/bin/cloud-init status --wait

# install prerequisites
sudo apt update
sudo apt install -y libpq-dev postgresql-14

# setup postgres
sudo -u postgres psql -c "ALTER ROLE postgres PASSWORD '${POSTGRES_PASSWORD}'"

# configure impulse postgres
if [[ "${POSTGRES_PURPOSE}" == "impulse" ]]; then
  sudo -u postgres psql -c "CREATE DATABASE impulse"
  sudo cp image_files/postgresql.conf /etc/postgresql/14/main/postgresql.conf
  sudo systemctl restart postgresql
  # database migration
  # diesel requirements
  sudo apt install -y build-essential libmysqlclient-dev libsqlite3-dev
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh
  chmod +x install_rustup.sh
  ./install_rustup.sh -y
  source "$HOME/.cargo/env"
  cargo install diesel_cli
  cp image_files/.env .
  diesel migration run
fi

# configure managed postgres
if [[ "${POSTGRES_PURPOSE}" == "managed" ]]; then
  ./release/setup_database --host localhost
fi

# modify firewall
ufw allow 5432
