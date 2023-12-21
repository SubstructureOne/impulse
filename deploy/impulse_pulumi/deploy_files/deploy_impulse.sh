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

sudo ufw allow 80

# install certbot
sudo snap install --classic certbot
sudo ln -sf /snap/bin/certbot /usr/bin/certbot
sudo certbot certonly -d "${IMPULSE_HOSTNAME}" -m "${EMAIL_ADDRESS}" --agree-tos -n --nginx


# start the services
sudo systemctl enable --now envoy.service
sudo systemctl enable --now prew.service
sudo systemctl enable --now impulse.timer

# ensure firewall is open on Postgresql port
sudo ufw allow 5432
