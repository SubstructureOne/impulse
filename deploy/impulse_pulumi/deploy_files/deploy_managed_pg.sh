#!/bin/bash

set -euxo pipefail

# Explicitly gate the block storage setup for now to prevent accidental 
# reconfiguration.
if [[ -n ${SETUP_BLOCK_STORAGE} ]]; then
  # Setup the block storage device as storage area for the "userdata" tablespace.
  # postgresql_managed.conf already has "userdata" as the default tablespace.
  sudo parted -s /dev/vdb mklabel gpt
  sudo parted -s /dev/vdb unit mib mkpart primary 0% 100%
  sudo mkfs.ext4 /dev/vdb1
  sudo mkdir /mnt/data
  echo | sudo tee -a /etc/fstab
  echo /dev/vdb1 /mnt/data ext4 defaults,noatime,nofail 0 0 | sudo tee -a /etc/fstab
  sudo mount /mnt/data
  sudo mkdir /mnt/data/postgres
  sudo chown postgres:postgres /mnt/data/postgres
  sudo -u postgres psql -c "CREATE TABLESPACE userdata LOCATION '/mnt/data/postgres'"
fi

  sudo fluent-gem install fluent-plugin-postgresql-csvlog --no-document
