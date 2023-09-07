#!/bin/bash

set -euxo pipefail

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
. ~/.nvm/nvm.sh

nvm install 18
nvm use node
npm install -g pm2
sudo apt install -y  nginx

git clone https://github.com/SubstructureOne/kestrelsite.git
cd kestrelsite
npm install
pm2 start npm -- start

sudo cp /home/ubuntu/nginx_default /etc/nginx/sites-available/default
sudo systemctl restart nginx
