#!/bin/bash

set -euxo pipefail

mkdir -p downloads/
curl -o downloads/fluent-apt-source.deb https://packages.treasuredata.com/lts/5/ubuntu/jammy/pool/contrib/f/fluent-lts-apt-source/fluent-lts-apt-source_2023.7.29-1_all.deb


packer build \
    -var-file=variables.hcl \
    "$@" \
    base_snapshot.pkr.hcl
