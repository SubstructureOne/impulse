#!/bin/bash

set -euxo pipefail

curl -o node.tar.xz https://nodejs.org/dist/v18.17.1/node-v18.17.1-linux-x64.tar.xz
tar xf node.tar.xz
rm -rf node
mv node-v18.17.1-linux-x64 node

if [ -d "kestrelsite" ]; then
  cd kestrelsite
  git pull
else
  git clone https://github.com/SubstructureOne/kestrelsite.git
  cd kestrelsite
fi
cp ../kestrelsite.env.local .env.local
NODE_PATH=/home/ubuntu/node PATH=$PATH:/home/ubuntu/node/bin /home/ubuntu/node/bin/npm install
NODE_PATH=/home/ubuntu/node PATH=$PATH:/home/ubuntu/node/bin /home/ubuntu/node/bin/npm run build

sudo cp /home/ubuntu/kestrelsite.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable kestrelsite --now
sudo cp /home/ubuntu/nginx_default /etc/nginx/sites-available/default
sudo systemctl restart nginx

sudo ufw allow 80
sudo ufw allow 443

sudo snap install --classic certbot
sudo ln -sf /snap/bin/certbot /usr/bin/certbot
sudo certbot --nginx -d "${SITE_HOSTNAME}" -m "${EMAIL_ADDRESS}" --agree-tos -n

