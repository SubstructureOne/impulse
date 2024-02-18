import os.path
from pathlib import Path

import pulumi
import ediri_vultr as vultr
import pulumi_command.remote
import pulumi_random

from .network import KestrelNetwork
from .postgres import ImpulsePgInstance, ManagedPgInstance
from .config import SSH_KEY_PATH


class SiteInstance:
    def __init__(
            self,
            config: pulumi.Config,
            network: KestrelNetwork,
            managed_pg_inst: ManagedPgInstance,
            impulse_pg_inst: ImpulsePgInstance,
    ):
        self.instance = vultr.Instance(
            "site_inst",
            snapshot_id=config.require("base_snapshot_id"),
            region=config.require("region"),
            plan=config.require("site_plan"),
            label=f"kestrelsite ({pulumi.get_stack()})",
            vpc_ids=[network.vpc.id],
            firewall_group_id=network.public_firewall.id,
        )
        self.reserved_ip = vultr.ReservedIp(
            "kestrel_site_ip",
            vultr.ReservedIpArgs(
                ip_type="v4",
                region=config.require("region"),
                instance_id=self.instance.id,
                label="reserved kestrel site IPv4",
            ),
            pulumi.ResourceOptions(
                protect=False,
            )
        )
        self.connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="ubuntu",
            private_key=Path(SSH_KEY_PATH).read_text()
        )
        self.connection_root = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="ubuntu",
            private_key=Path(SSH_KEY_PATH).read_text()
        )
        copy_nginx = pulumi_command.remote.CopyFile(
            "copy_nginx_config",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection_root,
                local_path="deploy_files/nginx_default",
                remote_path="/home/ubuntu/nginx_default",
                triggers=[os.path.getmtime("deploy_files/nginx_default")],
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        copy_service = pulumi_command.remote.CopyFile(
            "copy_kestrelsite_service",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection_root,
                local_path="deploy_files/kestrelsite.service",
                remote_path="/home/ubuntu/kestrelsite.service",
                triggers=[os.path.getmtime("deploy_files/kestrelsite.service")],
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        copy_setup = pulumi_command.remote.CopyFile(
            "copy_site_setup",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/deploy_website.sh",
                remote_path="/home/ubuntu/deploy_website.sh",
                triggers=[os.path.getmtime("deploy_files/deploy_website.sh")],
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        self.pgpass_encryption_key = pulumi.Output.secret(pulumi_random.RandomId(
            "pgpass_encryption_key",
            pulumi_random.RandomIdArgs(byte_length=32),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        ))
        write_env_command = pulumi.Output.format(
            """
cat <<EOT >/home/ubuntu/kestrelsite.env.local
NEXT_PUBLIC_SUPABASE_URL={0}
NEXT_PUBLIC_SUPABASE_ANON_KEY={1}
NEXT_PUBLIC_PREVIEW_MODE_DISABLED=1
POSTGRES_HOST={2}
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD={3}
POSTGRES_DATABASE=impulse
MANAGED_PG_HOST={4}
MANAGED_PG_PORT=5432
MANAGED_PG_USER=postgres
MANAGED_PG_PASSWORD={5}
STRIPE_SECRET_KEY={6}
STRIPE_WEBHOOK_SECRET={7}
STRIPE_FUND_ACCOUNT_PRICE_ID={8}
STRIPE_FUND_ACCOUNT_SUCCESS_URL={9}
STRIPE_FUND_ACCOUNT_CANCEL_URL={10}
PGPASS_KEY_B64={11}
NEXT_PUBLIC_BASE_URL=https://{12}
EOT
            """,
            config.require("supabase_url"),
            config.require("supabase_anon_key"),
            impulse_pg_inst.instance.internal_ip,
            impulse_pg_inst.password.result,
            managed_pg_inst.instance.internal_ip,
            managed_pg_inst.password.result,
            config.require_secret("stripe_secret_key"),
            config.require_secret("stripe_webhook_secret"),
            config.require_secret("stripe_fund_price_id"),
            config.require("stripe_fund_success_url"),
            config.require("stripe_fund_cancel_url"),
            self.pgpass_encryption_key.b64_std,
            config.require("site_hostname"),
        )
        write_env = pulumi_command.remote.Command(
            "write_kestrelsite_env_file",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create=write_env_command,
            ),
            pulumi.ResourceOptions(
                parent=self.instance,
            )
        )
        pulumi_command.remote.Command(
            "configure_site",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create=f"""EMAIL_ADDRESS="{config.require("email_address")}" SITE_HOSTNAME="{config.require("site_hostname")}" bash /home/ubuntu/deploy_website.sh""",
                triggers=[copy_setup, write_env_command],
            ),
            pulumi.ResourceOptions(
                depends_on=[copy_nginx, copy_setup, write_env, self.reserved_ip, impulse_pg_inst.configured, network.public_firewall],
                parent=self.instance,
            )
        )
