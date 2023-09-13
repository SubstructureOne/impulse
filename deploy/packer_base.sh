#!/bin/bash

set -euxo pipefail

packer build \
    -var-file=variables.hcl \
    "$@" \
    base_snapshot.pkr.hcl
