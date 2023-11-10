import ipaddress

import pulumi
import ediri_vultr as vultr


class KestrelNetwork:
    def __init__(self, config: pulumi.Config):
        self.vpc = vultr.Vpc(
            "kestrel-vpc",
            description="Kestrel VPC",
            region=config.require("region")
        )
        self.top_domain = vultr.DnsDomain("kdeploy.com", domain="kdeploy.com")

        # VPC rules
        self.private_firewall = vultr.FirewallGroup(
            "vpc_private",
            vultr.FirewallGroupArgs(
                description="Kestrel VPC firewall group"
            ),
        )
        # public-facing rules
        self.public_firewall = vultr.FirewallGroup(
            "firewall_public",
            vultr.FirewallGroupArgs(
                description="Kestrel 'public' firewall group"
            )
        )
        # convert to form "192.168.0.0/255.255.255.0", which can be parsed by
        # ipaddress.IPv4Network
        subnet_str = pulumi.Output.format(
            "{0}/{1}",
            self.vpc.v4_subnet,
            self.vpc.v4_subnet_mask
        )
        vpc_subnet = subnet_str.apply(ipaddress.IPv4Network)
        vultr.FirewallRule(
            "pg_vpc",
            vultr.FirewallRuleArgs(
                firewall_group_id=self.private_firewall.id,
                protocol="tcp",
                ip_type="v4",
                subnet=self.vpc.v4_subnet,
                subnet_size=vpc_subnet.prefixlen,
                port="5432",
                notes="allow PG connections from VPC",
            ),
            pulumi.ResourceOptions(
                parent=self.private_firewall,
            )
        )

        def handle_trusted_ips(trusted_ips):
            for ind, trusted_ip in enumerate(trusted_ips):
                vultr.FirewallRule(
                    f"pg_trusted_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.private_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=trusted_ip,
                        subnet_size=32,
                        port="5432",
                        notes=f"allow trusted IP {ind} to connect to Postgres",
                    ),
                    pulumi.ResourceOptions(
                        parent=self.private_firewall,
                        delete_before_replace=True,
                        aliases=[pulumi.Alias(parent=None)]
                    ),
                )
                vultr.FirewallRule(
                    f"ssh_trusted_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.private_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=trusted_ip,
                        subnet_size=32,
                        port="22",
                        notes=f"allow SSH from trusted {ind}",
                    ),
                    pulumi.ResourceOptions(
                        parent=self.private_firewall,
                    ),
                )
                vultr.FirewallRule(
                    f"ssh_public_trusted_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.public_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=trusted_ip,
                        subnet_size=32,
                        port="22",
                        notes=f"allow SSH from trusted {ind}",
                    ),
                    pulumi.ResourceOptions(
                        parent=self.public_firewall,
                    ),
                )
        config.require_secret_object("trusted_ips").apply(handle_trusted_ips)

        vultr.FirewallRule(
            "icmp_allow",
            vultr.FirewallRuleArgs(
                firewall_group_id=self.private_firewall.id,
                protocol="icmp",
                ip_type="v4",
                subnet="0.0.0.0",
                subnet_size=0,
                notes="allow ICMP from any host",
            ),
            pulumi.ResourceOptions(
                parent=self.private_firewall,
            )
        )

        def handle_public_ips(public_ips):
            for ind, public_ip in enumerate(public_ips):
                if public_ip == "0.0.0.0":
                    subnet_size = 0
                else:
                    subnet_size = 32
                vultr.FirewallRule(
                    f"public_http_access_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.public_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=public_ip,
                        subnet_size=subnet_size,
                        port="80",
                        notes=f"allow HTTP from public {ind}",
                    )
                )
                vultr.FirewallRule(
                    f"public_https_access_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.public_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=public_ip,
                        subnet_size=subnet_size,
                        port="443",
                        notes=f"allow HTTP from public {ind}",
                    )
                )
                vultr.FirewallRule(
                    f"public_pg_access_{ind}",
                    vultr.FirewallRuleArgs(
                        firewall_group_id=self.public_firewall.id,
                        protocol="tcp",
                        ip_type="v4",
                        subnet=public_ip,
                        subnet_size=subnet_size,
                        port="5432",
                        notes=f"allow PG fromm public {ind}",
                    )
                )
        config.require_secret_object("public_ips").apply(handle_public_ips)
