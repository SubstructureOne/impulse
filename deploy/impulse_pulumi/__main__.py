"""Pulumi configuration for deploy Impulse"""

import pulumi
import ediri_vultr as vultr

config = pulumi.Config()
vpc = vultr.Vpc(
    "kestrel-vpc",
    description="kestrel vpc",
    region=config.require("region")
)
impulse_inst = vultr.Instance(
    "impulse",
    snapshot_id=config.require("impulse_snapshot_id"),
    region=config.require("region"),
    plan=config.require("impulse_plan"),
    label=config.require("impulse_instance_label"),
    vpc_ids=[vpc.id],
)
managed_inst = vultr.Instance(
    "managed_pg",
    snapshot_id=config.require("managed_snapshot_id"),
    region=config.require("region"),
    plan=config.require("managed_plan"),
    label=config.require("managed_instance_label"),
    vpc_ids=[vpc.id],
)
impulse_pg_inst = vultr.Instance(
    "impulse_pg",
    snapshot_id=config.require("impulse_pg_snapshot_id"),
    region=config.require("region"),
    plan=config.require("impulse_pg_plan"),
    label=config.require("impulse_pg_instance_label"),
    vpc_ids=[vpc.id],
)

pulumi.export('impulse_instance_id', impulse_inst.id)
pulumi.export('managed_pg_instance_id', managed_inst.id)
pulumi.export('impulse_pg_instance_id', impulse_pg_inst.id)
