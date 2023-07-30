#!/bin/bash

set -euxo pipefail

cd /setup

ENVOYDOWNLOAD="https://github.com/envoyproxy/envoy/releases/download/v1.26.3/envoy-contrib-1.26.3-linux-x86_64"

# install prerequisites
# sudo apt update
sudo apt install -y libpq-dev postgresql-14

# setup postgres
sudo -u postgres psql -c "ALTER ROLE postgres PASSWORD '${POSTGRES_PASSWORD}'"
sudo cp image_files/postgresql.conf /etc/postgresql/14/main/postgresql.conf
sudo systemctl restart postgresql

sudo mkdir -p /opt/impulse/bin
sudo mkdir /opt/impulse/etc
sudo mkdir -p /opt/envoy/bin
sudo mkdir /opt/envoy/etc
sudo useradd prew

# install envoy
curl -L ${ENVOYDOWNLOAD} > envoy
chmod +x envoy
sudo mv envoy /opt/envoy/bin/envoy
sudo mv image_files/envoy-postgres.yaml /opt/envoy/etc/envoy-postgres.yaml
sudo chown -R root:root /opt/envoy/

# install impulse binaries
sudo mv release/* /opt/impulse/bin/
sudo cp image_files/.env /opt/impulse/bin/
sudo cp image_files/prew.toml /opt/impulse/etc
sudo chown -R root:root /opt/impulse/

# generate self-signed certificate for envoy to use for SSL connections
openssl req -new -newkey rsa:4096 -subj "/CN=Widgets Inc/C=US/ST=Ohio/L=Columbus/O=Widgets Inc/OU=Some Unit" -nodes -keyout ssl-cert.key -out ssl-cert.csr
openssl x509 -req -sha256 -days 365 -in ssl-cert.csr -signkey ssl-cert.key -out ssl-cert.pem
sudo cp ssl-cert.key /etc/ssl/private/
sudo cp ssl-cert.pem /etc/ssl/certs/
sudo chown -R prew:prew /etc/ssl/private/

# install systemd services
sudo cp image_files/envoy.service image_files/impulse.service image_files/impulse.timer image_files/prew.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now envoy.service
sudo systemctl enable --now prew.service
sudo systemctl enable --now impulse.timer
