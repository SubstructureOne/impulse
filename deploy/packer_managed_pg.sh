packer build \
    -var-file=variables.hcl \
    -var 'postgres_purpose=managed' \
    -var 'snapshot_name=pg-managed' \
    postgresql.pkr.hcl
