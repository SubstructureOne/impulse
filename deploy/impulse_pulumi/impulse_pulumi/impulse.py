import pulumi
import pulumi_command
import ediri_vultr as vultr

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
            label=config.require("impulse_instance_label"),
            vpc_ids=[network.vpc.id],
        )
        with open(config.require("ssh_key_path"), "r") as fp:
            private_key = fp.read()
        connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="root",
            private_key=private_key,
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
            managed_inst.instance.internal_ip,
            managed_inst.password,
            impulse_pg_inst.instance.internal_ip,
            impulse_pg_inst.password,
        )
        write_env = pulumi_command.remote.Command(
            "write_env_file",
            connection=connection,
            create=dotenv_write_cmd,
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
            connection=connection,
            create=prew_toml_write_cmd,
        )
        # run migration
        # pulumi_command.remote.Command(
        #     "migrate_database",
        #     pulumi_command.remote.CommandArgs(
        #         connection=connection,
        #         create="env --chdir=/opt/impulse/bin diesel migration run",
        #     ),
        #     pulumi.ResourceOptions(
        #         depends_on=[write_env],
        #     )
        # )
