#!/bin/bash

set -euxo pipefail

cd /opt/impulse/bin

# configure impulse database
. .env
psql -c "CREATE DATABASE impulse" "postgres://${KESTREL_DB_USER}:${KESTREL_DB_PASSWORD}@${KESTREL_DB_HOST}:${KESTREL_DB_PORT}/${KESTREL_DB_USER}" || :
rm -rf migrations/
tar xzvf migrations.tar.gz
/root/.cargo/bin/diesel migration run

# configure managed database (refers to environment vars)
./setup_database

# start the services
sudo systemctl enable --now envoy.service
sudo systemctl enable --now prew.service
sudo systemctl enable --now impulse.timer

# ensure firewall is open on Postgresql port
ufw allow 5432
