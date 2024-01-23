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

# copy binaries
sudo systemctl stop prew.service
sudo cp /root/prew /opt/impulse/bin/prew
sudo systemctl stop impulse.service
sudo cp /root/impulse /opt/impulse/bin/impulse

sudo ufw allow 80

# install certbot
sudo snap install --classic certbot
sudo ln -sf /snap/bin/certbot /usr/bin/certbot
sudo certbot certonly -d "${IMPULSE_HOSTNAME}" -m "${EMAIL_ADDRESS}" --agree-tos -n --nginx

# install envoy certificates
sudo cp "/etc/letsencrypt/live/${IMPULSE_HOSTNAME}/fullchain.pem" /etc/ssl/certs/ssl-cert.pem
sudo chown prew:prew /etc/ssl/certs/ssl-cert.pem
sudo chmod 644 /etc/ssl/certs/ssl-cert.pem
sudo cp "/etc/letsencrypt/live/${IMPULSE_HOSTNAME}/privkey.pem" /etc/ssl/private/ssl-cert.key
sudo chown prew:prew /etc/ssl/private/ssl-cert.key
sudo chmod 600 /etc/ssl/private/ssl-cert.key

# deploy envoy configuration file
cat /root/envoy-postgres.yaml.tmpl | envsubst > envoy-postgres.yaml
sudo cp envoy-postgres.yaml /opt/envoy/etc/envoy-postgres.yaml

# start the services
sudo systemctl enable envoy.service
sudo systemctl restart envoy.service
sudo systemctl enable prew.service
sudo systemctl restart prew.service
sudo systemctl enable impulse.timer
sudo systemctl restart impulse.timer

# ensure firewall is open on Postgresql port
sudo ufw allow ${ENVOY_PORT}
