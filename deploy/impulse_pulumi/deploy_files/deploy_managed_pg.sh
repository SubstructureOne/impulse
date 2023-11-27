#!/bin/bash

set -euxo pipefail

sudo parted -s /dev/vdb mklabel gpt
sudo parted -s /dev/vdb unit mib mkpart primary 0% 100%
sudo mkfs.ext4 /dev/vdb1
sudo mkdir /mnt/data
sudo echo >> /etc/fstab
sudo echo /dev/vdb1 /mnt/data ext4 defaults,noatime,nofail 0 0 >> /etc/fstab
sudo mount /mnt/data
