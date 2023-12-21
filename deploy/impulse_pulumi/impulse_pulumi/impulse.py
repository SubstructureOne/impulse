import os
import os.path
import pulumi
import pulumi_command
import ediri_vultr as vultr

from .config import SSH_KEY_PATH
from .network import KestrelNetwork
from .postgres import ManagedPgInstance, ImpulsePgInstance


class ImpulseInstance:
    def __init__(
            self,
            config: pulumi.Config,
            network: KestrelNetwork,
            managed_inst: ManagedPgInstance,
            impulse_pg_inst: ImpulsePgInstance,
    ):
        self.instance = vultr.Instance(
            "impulse",
            snapshot_id=config.require("impulse_snapshot_id"),
            region=config.require("region"),
            plan=config.require("impulse_plan"),
            label=f"impulse ({pulumi.get_stack()})",
            vpc_ids=[network.vpc.id],
            firewall_group_id=network.public_firewall.id,
        )
        self.reserved_ip = vultr.ReservedIp(
            "impulse_ip",
            vultr.ReservedIpArgs(
                ip_type="v4",
                region=config.require("region"),
                instance_id=self.instance.id,
                label="reserved impulse IPv4",
            ),
            pulumi.ResourceOptions(
                protect=True,
            )
        )
        with open(SSH_KEY_PATH, "r") as fp:
            private_key = fp.read()
        connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="root",
            private_key=private_key,
        )
        # "{{" in an f-string (or to Output.format) becomes the literal "{",
        # and likewise "}}" becomes "}". In the following, {blah} is passed to and
        # formatted by Pulumi when the outputs are known, and {{blah}} is written
        # verbatim to the output file as "{blah}", where "${blah}" is then processed by
        # the library reading the .env file.
        # Because this is written using a shell environment, the ${...} variable
        # references also need to be escaped to prevent attempting to interpolate
        # them at write-time.
        dotenv_write_cmd = pulumi.Output.format(
            """
cat <<EOT >/opt/impulse/bin/.env
MANAGED_DB_HOST={0}
MANAGED_DB_PORT=5432
MANAGED_DB_USER=postgres
MANAGED_DB_PASSWORD={1}
KESTREL_DB_HOST={2}
KESTREL_DB_PORT=5432
KESTREL_DB_USER=postgres
KESTREL_DB_PASSWORD={3}
DATABASE_URL=postgres://\\${{KESTREL_DB_USER}}:\\${{KESTREL_DB_PASSWORD}}@\\${{KESTREL_DB_HOST}}:\\${{KESTREL_DB_PORT}}/impulse
EOT
        """,
            managed_inst.instance.internal_ip,
            managed_inst.password.result,
            impulse_pg_inst.instance.internal_ip,
            impulse_pg_inst.password.result,
        )
        write_env = pulumi_command.remote.Command(
            "write_env_file",
            pulumi_command.remote.CommandArgs(
                connection=connection,
                create=dotenv_write_cmd,
                triggers=[
                    managed_inst.instance.internal_ip,
                    managed_inst.password.result,
                    impulse_pg_inst.instance.internal_ip,
                    impulse_pg_inst.password.result,
                ]
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        prew_toml_write_cmd = pulumi.Output.format(
            """
cat <<EOT >/opt/impulse/etc/prew.toml
bind_addr = "0.0.0.0:6432"
server_addr = "{}:5432"
EOT
systemctl restart prew
""",
            managed_inst.instance.internal_ip,
        )
        pulumi_command.remote.Command(
            "write_prewtoml",
            pulumi_command.remote.CommandArgs(
                connection=connection,
                create=prew_toml_write_cmd,
                triggers=[managed_inst.instance.internal_ip],
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        # run migration
        tarball_path = os.path.join(os.getcwd(), "migrations.tar.gz")
        migrations_dir_rel = "../.."
        create_tarball = pulumi_command.local.Command(
            "create_migration_tarball",
            pulumi_command.local.CommandArgs(
                create=f"""bash -c "pushd {migrations_dir_rel}; tar czvf {tarball_path} migrations/" """,
                triggers=[sorted(os.listdir(migrations_dir_rel))],
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        copy_tarball = pulumi_command.remote.CopyFile(
            "copy_migrations_tarball",
            pulumi_command.remote.CopyFileArgs(
                connection=connection,
                local_path=tarball_path,
                remote_path="/opt/impulse/bin/migrations.tar.gz",
                triggers=[create_tarball],
            ),
            pulumi.ResourceOptions(
                depends_on=[create_tarball],
                parent=self.instance,
            )
        )
        deploy_impulse_script = "deploy_files/deploy_impulse.sh"
        copy_deploy_script = pulumi_command.remote.CopyFile(
            "copy_deploy_impulse",
            pulumi_command.remote.CopyFileArgs(
                connection=connection,
                local_path=deploy_impulse_script,
                remote_path="/root/deploy_impulse.sh",
                triggers=[os.path.getmtime(deploy_impulse_script)]
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        pulumi_command.remote.Command(
            "run_deploy_impulse",
            pulumi_command.remote.CommandArgs(
                connection=connection,
                create=f"""EMAIL_ADDRESS="{config.require("email_address")}" IMPULSE_HOSTNAME="{config.require("impulse_hostname")}" bash /root/deploy_impulse.sh""",
                triggers=[copy_deploy_script, copy_tarball],
            ),
            pulumi.ResourceOptions(
                depends_on=[copy_deploy_script, copy_tarball],
                parent=self.instance,
            )
        )
