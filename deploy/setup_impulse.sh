#!/bin/bash

set -euxo pipefail

cd /setup

ENVOYDOWNLOAD="https://github.com/envoyproxy/envoy/releases/download/v1.26.3/envoy-contrib-1.26.3-linux-x86_64"

# wait for cloud-init to finish
/usr/bin/cloud-init status --wait

# disable postgres
sudo systemctl stop postgresql
sudo systemctl disable postgresql

# install diesel_cli
# FIXME: overkill to install rust and compile just to get this one binary.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh
chmod +x install_rustup.sh
./install_rustup.sh -y
source "$HOME/.cargo/env"
cargo install diesel_cli --no-default-features --features postgres

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
sudo chown -R root:root /opt/impulse/

# generate self-signed certificate for envoy to use for SSL connections
openssl req -new -newkey rsa:4096 -subj "/CN=Widgets Inc/C=US/ST=Ohio/L=Columbus/O=Widgets Inc/OU=Some Unit" -nodes -keyout ssl-cert.key -out ssl-cert.csr
openssl x509 -req -sha256 -days 365 -in ssl-cert.csr -signkey ssl-cert.key -out ssl-cert.pem
sudo cp ssl-cert.key /etc/ssl/private/
sudo cp ssl-cert.pem /etc/ssl/certs/
sudo chown -R prew:prew /etc/ssl/private/

# install systemd services
# These services cannot be fully configured until the deployment specifics
# are defined, so don't start them yet.
sudo cp image_files/envoy.service image_files/impulse.service image_files/impulse.timer image_files/prew.service /etc/systemd/system/
sudo systemctl daemon-reload

# modify firewall to allow connections to prew
ufw allow 5432
