"""Pulumi configuration for deploy Impulse"""

import pulumi
import pulumi_command
import ediri_vultr as vultr

config = pulumi.Config()
top_domain = vultr.DnsDomain("kdeploy.com", domain="kdeploy.com")
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
# Reminder, "{{" in an f-string (or to Output.format) becomes the literal "{",
# and likewise "}}" becomes "}". In the following, {blah} is immediately
# formatted as part of the f-string. {{blah}} is passed to and formatted by
# Pulumi when the outputs are known. And {{{{blah}}}} is written verbatim
# to the output file as "{blah}", where "${blah}" is then processed by the
# library reading the .env file.
dotenv_write_cmd = pulumi.Output.format(
    f"""
cat <<EOT >/opt/impulse/bin/.env
MANAGED_DB_HOST={{0}}
MANAGED_DB_PORT=5432
MANAGED_DB_USER=postgres
MANAGED_DB_PASSWORD={config.require("managed_db_password")}
KESTREL_DB_HOST={{1}}
KESTREL_DB_PORT=5432
KESTREL_DB_USER=postgres
KESTREL_DB_PASSWORD={config.require("kestrel_db_password")}
DATABASE_URL=postgres://${{{{MANAGED_DB_USER}}}}:${{{{MANAGED_DB_PASSWORD}}}}@${{{{MANAGED_DB_HOST}}}}:${{{{MANAGED_DB_PORT}}}}/impulse
EOT
""",
    managed_inst.main_ip,
    impulse_inst.main_ip,
)
connection = pulumi_command.remote.ConnectionArgs(
    host=impulse_inst.main_ip,
    user="root",
    password=impulse_inst.default_password
)
pulumi_command.remote.Command(
    "write_env_file",
    connection=connection,
    create=dotenv_write_cmd,
)

pulumi.export('impulse_instance_id', impulse_inst.id)
pulumi.export('managed_pg_instance_id', managed_inst.id)
pulumi.export('impulse_pg_instance_id', impulse_pg_inst.id)
pulumi.export("impulse_ip", impulse_inst.main_ip)
pulumi.export("env_file", dotenv_write_cmd)
