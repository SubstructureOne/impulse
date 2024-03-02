set -euxo pipefail

# wait for cloud-init to finish
/usr/bin/cloud-init status --wait

cd /setup

passwd --delete ubuntu

# prep to be able to install fluentd
apt install -y ./fluent-apt-source.deb

apt update
apt install -y libpq-dev postgresql-14 build-essential nginx fluent-package
# !!!
# Vultr by default modifies sshd_config to allow the root user to login
# via the root password (autocreated at initialization). The use of
# --force-confnew here will overwrite that back to the default of only
# allowing root to login via key if there's an updated version of the
# openssh-server package.
# !!!
DEBIAN_FRONTEND=noninteractive apt dist-upgrade -y -o Dpkg::Options::="--force-confdef" -o Dpkg::Options::="--force-confnew"
cp /root/.ssh/authorized_keys /home/ubuntu/.ssh/
sudo shutdown -r now
