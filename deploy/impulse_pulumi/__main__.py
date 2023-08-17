"""Pulumi configuration for deploy Impulse"""

import pulumi
import ediri_vultr as vultr

config = pulumi.Config()
impulse_inst = vultr.Instance(
    "impulse-dev",
    snapshot_id=config.require("snapshot_id"),
    region=config.require("region"),
    plan=config.require("plan"),
    label=config.require("instance_label"),
)

pulumi.export('instance_id', impulse_inst.id)
