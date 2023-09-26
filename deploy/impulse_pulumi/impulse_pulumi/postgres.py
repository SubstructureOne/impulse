from pathlib import Path

import pulumi
import pulumi_command
import pulumi_random
import ediri_vultr as vultr

from .config import SSH_KEY_PATH
from .network import KestrelNetwork


class ManagedPgInstance:
    def __init__(
            self,
            config: pulumi.Config,
            network: KestrelNetwork,
    ):
        self.instance = vultr.Instance(
            "managed_pg",
            vultr.InstanceArgs(
                snapshot_id=config.require("base_snapshot_id"),
                region=config.require("region"),
                plan=config.require("managed_pg_plan"),
                label=config.require("managed_pg_instance_label"),
                vpc_ids=[network.vpc.id],
                firewall_group_id=network.private_firewall.id,
            ),
        )
        self.password = pulumi_random.RandomPassword(
            "managed_pg_password",
            pulumi_random.RandomPasswordArgs(
                length=16,
                numeric=True,
                special=False,
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        self.connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="root",
            private_key=Path(SSH_KEY_PATH).read_text(),
        )
        setpass = pulumi_command.remote.Command(
            "set_managed_pg_password",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create=pulumi.Output.format(
                    """sudo -u postgres psql -c "ALTER ROLE postgres PASSWORD '{0}'" """,
                    self.password.result
                )
            ),
            pulumi.ResourceOptions(
                parent=self.instance
            )
        )
        postgresql_conf = pulumi_command.remote.CopyFile(
            "managed_pg_postgresql_conf",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/postgresql.conf",
                remote_path="/etc/postgresql/14/main/postgresql.conf",
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        pg_hba_conf = pulumi_command.remote.CopyFile(
            "managed_pg_hba_conf",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/pg_hba.conf",
                remote_path="/etc/postgresql/14/main/pg_hba.conf",
            ),
            pulumi.ResourceOptions(
                parent=self.instance
            )
        )
        pulumi_command.remote.Command(
            "restart_managed_postgresql",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create="""bash -c "ufw allow 5432 && systemctl restart postgresql" """
            ),
            pulumi.ResourceOptions(
                depends_on=[setpass, postgresql_conf, pg_hba_conf],
                parent=self.instance,
            ),
        )


class ImpulsePgInstance:
    def __init__(
            self,
            config: pulumi.Config,
            network: KestrelNetwork,
    ):
        self.instance = vultr.Instance(
            "impulse_pg",
            snapshot_id=config.require("base_snapshot_id"),
            region=config.require("region"),
            plan=config.require("impulse_pg_plan"),
            label="impulse_pg",
            vpc_ids=[network.vpc.id],
            firewall_group_id=network.private_firewall.id,
        )
        self.password = pulumi_random.RandomPassword(
            "impulse_pg_password",
            pulumi_random.RandomPasswordArgs(
                length=16,
                numeric=True,
                special=False,
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        self.connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="root",
            private_key=Path(SSH_KEY_PATH).read_text(),
        )
        setpass = pulumi_command.remote.Command(
            "set_impulse_pg_password",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create=pulumi.Output.format(
                    """sudo -u postgres psql -c "ALTER ROLE postgres PASSWORD '{0}'" """,
                    self.password.result,
                )
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        postgresql_conf = pulumi_command.remote.CopyFile(
            "impulse_pg_postgresql_conf",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/postgresql.conf",
                remote_path="/etc/postgresql/14/main/postgresql.conf",
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        pg_hba_conf = pulumi_command.remote.CopyFile(
            "impulse_pg_hba_conf",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/pg_hba.conf",
                remote_path="/etc/postgresql/14/main/pg_hba.conf",
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        self.configured = pulumi_command.remote.Command(
            "restart_impulse_postgresql",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create="""bash -c "ufw allow 5432 && systemctl restart postgresql" """
            ),
            pulumi.ResourceOptions(
                depends_on=[setpass, postgresql_conf, pg_hba_conf],
                parent=self.instance,
            ),
        )
