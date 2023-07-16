#!/bin/bash

set -euxo pipefail

cd /setup

ENVOYDOWNLOAD="https://github.com/envoyproxy/envoy/releases/download/v1.26.3/envoy-contrib-1.26.3-linux-x86_64"

sudo mkdir -p /opt/impulse/bin
sudo mkdir -p /opt/envoy/bin

# install envoy
curl -L ${ENVOYDOWNLOAD} > envoy
sudo mv envoy /opt/envoy/bin/envoy
sudo mv image_files/envoy-postgres.yaml /opt/envoy/etc/envoy-postgres.yaml
sudo chown -R root:root /opt/envoy/

# install impulse binaries
sudo mv release/* /opt/impulse/bin/
sudo chown -R root:root /opt/impulse/

# install systemd services
sudo cp image_files/envoy.service images_files/impulse.service image_files/impulse.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now envoy.service
sudo systemctl enable --now impulse.timer
