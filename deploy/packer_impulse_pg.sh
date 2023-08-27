packer build \
    -var-file=variables.hcl \
    -var 'postgres_purpose=impuulse' \
    -var 'snapshot_name=pg-impulse' \
    postgresql.pkr.hcl
