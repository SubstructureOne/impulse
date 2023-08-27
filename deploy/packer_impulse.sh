packer build \
    -var-file=variables.hcl \
    -var="snapshot_name=impulse" \
    impulse.pkr.hcl
