import os.path
from pathlib import Path

import pulumi
import ediri_vultr as vultr
import pulumi_command.remote

from .network import KestrelNetwork


class SiteInstance:
    def __init__(self, config: pulumi.Config, network: KestrelNetwork):
        self.instance = vultr.Instance(
            "site_inst",
            snapshot_id=config.require("base_snapshot_id"),
            region=config.require("region"),
            plan=config.require("site_plan"),
            label="kesetrelsite",
            vpc_ids=[network.vpc.id],
        )
        self.connection = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="ubuntu",
            private_key=Path(config.require("ssh_key_path")).read_text()
        )
        self.connection_root = pulumi_command.remote.ConnectionArgs(
            host=self.instance.main_ip,
            user="ubuntu",
            private_key=Path(config.require("ssh_key_path")).read_text()
        )
        copy_nginx = pulumi_command.remote.CopyFile(
            "copy_nginx_config",
            connection=self.connection_root,
            local_path="deploy_files/nginx_default",
            remote_path="/home/ubuntu/nginx_default",
        )
        copy_service = pulumi_command.remote.CopyFile(
            "copy_kestrelsite_service",
            connection=self.connection_root,
            local_path="deploy_files/kestrelsite.service",
            remote_path="/home/ubuntu/kestrelsite.service",
            triggers=[os.path.getmtime("deploy_files/kestrelsite.service")],
        )
        copy_setup = pulumi_command.remote.CopyFile(
            "copy_site_setup",
            pulumi_command.remote.CopyFileArgs(
                connection=self.connection,
                local_path="deploy_files/deploy_website.sh",
                remote_path="/home/ubuntu/deploy_website.sh",
                triggers=[os.path.getmtime("deploy_files/deploy_website.sh")],
            ),
        )
        pulumi_command.remote.Command(
            "configure_site",
            pulumi_command.remote.CommandArgs(
                connection=self.connection,
                create="bash /home/ubuntu/deploy_website.sh",
            ),
            pulumi.ResourceOptions(
                depends_on=[copy_nginx, copy_setup],
                parent=copy_setup,
            )
        )
