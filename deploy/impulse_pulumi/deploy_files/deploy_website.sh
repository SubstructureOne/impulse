#!/bin/bash

set -euxo pipefail

sudo apt install -y nginx

wget https://nodejs.org/dist/v18.17.1/node-v18.17.1-linux-x64.tar.xz
tar xvf node-v18.17.1-linux-x64.tar.xz
rm -rf node
mv node-v18.17.1-linux-x64 node

if [ -d "kestrelsite" ]; then
  cd kestrelsite
  git pull
else
  git clone https://github.com/SubstructureOne/kestrelsite.git
  cd kestrelsite
fi
NODE_PATH=/home/ubuntu/node /home/ubuntu/node/bin/npm install

sudo cp /home/ubuntu/kestrelsite.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable kestrelsite --now
sudo cp /home/ubuntu/nginx_default /etc/nginx/sites-available/default
sudo systemctl restart nginx
