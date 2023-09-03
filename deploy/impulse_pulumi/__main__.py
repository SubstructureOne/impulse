"""Pulumi configuration for deploy Impulse"""

import pulumi
import pulumi_command
import ipaddress
import ediri_vultr as vultr

config = pulumi.Config()
top_domain = vultr.DnsDomain("kdeploy.com", domain="kdeploy.com")
vpc = vultr.Vpc(
    "kestrel-vpc",
    description="kestrel vpc",
    region=config.require("region")
)
privatefirewall = vultr.FirewallGroup("vpc_private")
# convert to form "192.168.0.0/255.255.255.0", which can be parsed by
# ipaddress.IPv4Network
subnet_str = pulumi.Output.format("{0}/{1}", vpc.v4_subnet, vpc.v4_subnet_mask)
vpc_subnet = subnet_str.apply(ipaddress.IPv4Network)
vultr.FirewallRule(
    firewall_group_id=privatefirewall.id,
    resource_name="pg_vpn_only",
    protocol="tcp",
    ip_type="v4",
    subnet=vpc.v4_subnet,
    subnet_size=vpc_subnet.prefixlen,
    port="5432",
    notes="allow PG connections only from VPC"
)
vultr.FirewallRule(
    firewall_group_id=privatefirewall.id,
    resource_name="pg_allow_ssh_all",
    protocol="tcp",
    ip_type="v4",
    subnet="0.0.0.0",
    subnet_size=0,
    port="22",
    notes="allow SSH from any host",
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
    firewall_group_id=privatefirewall.id,
)
impulse_pg_inst = vultr.Instance(
    "impulse_pg",
    snapshot_id=config.require("impulse_pg_snapshot_id"),
    region=config.require("region"),
    plan=config.require("impulse_pg_plan"),
    label=config.require("impulse_pg_instance_label"),
    vpc_ids=[vpc.id],
    firewall_group_id=privatefirewall.id,
)
# Reminder, "{{" in an f-string (or to Output.format) becomes the literal "{",
# and likewise "}}" becomes "}". In the following, {blah} is passed to and
# formatted by Pulumi when the outputs are known, and {{blah}} is written
# verbatim to the output file as "{blah}", where "${blah}" is then processed by
# the library reading the .env file.
dotenv_write_cmd = pulumi.Output.format(
    """
cat <<EOT >/opt/impulse/bin/.env
MANAGED_DB_HOST={0}
MANAGED_DB_PORT=5432
MANAGED_DB_USER=postgres
MANAGED_DB_PASSWORD={1}
KESTREL_DB_HOST={{2}}
KESTREL_DB_PORT=5432
KESTREL_DB_USER=postgres
KESTREL_DB_PASSWORD={3}
DATABASE_URL=postgres://${{MANAGED_DB_USER}}:${{MANAGED_DB_PASSWORD}}@${{MANAGED_DB_HOST}}:${{MANAGED_DB_PORT}}/impulse
EOT
""",
    managed_inst.internal_ip,
    config.require_secret("managed_db_password"),
    impulse_inst.main_ip,
    config.require_secret("kestrel_db_password"),
)
prew_toml_write_cmd = pulumi.Output.format(
    """
cat <<EOT >/opt/impulse/etc/prew.toml
bind_addr = "0.0.0.0:6432"
server_addr = "{}:5432"
EOT
systemctl restart prew
""",
    managed_inst.internal_ip,
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
pulumi_command.remote.Command(
    "write_prewtoml",
    connection=connection,
    create=prew_toml_write_cmd,
)

pulumi.export('impulse_instance_id', impulse_inst.id)
pulumi.export('managed_pg_instance_id', managed_inst.id)
pulumi.export('impulse_pg_instance_id', impulse_pg_inst.id)
pulumi.export("impulse_ip", impulse_inst.main_ip)
pulumi.export("env_file", dotenv_write_cmd)
