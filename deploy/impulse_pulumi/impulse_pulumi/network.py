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
        self.private_firewall = vultr.FirewallGroup("vpc_private")
        # convert to form "192.168.0.0/255.255.255.0", which can be parsed by
        # ipaddress.IPv4Network
        subnet_str = pulumi.Output.format(
            "{0}/{1}",
            self.vpc.v4_subnet,
            self.vpc.v4_subnet_mask
        )
        vpc_subnet = subnet_str.apply(ipaddress.IPv4Network)
        vultr.FirewallRule(
            firewall_group_id=self.private_firewall.id,
            resource_name="pg_vpn_only",
            protocol="tcp",
            ip_type="v4",
            subnet=self.vpc.v4_subnet,
            subnet_size=vpc_subnet.prefixlen,
            port="5432",
            notes="allow PG connections only from VPC"
        )
        vultr.FirewallRule(
            firewall_group_id=self.private_firewall.id,
            resource_name="pg_allow_ssh_all",
            protocol="tcp",
            ip_type="v4",
            subnet="0.0.0.0",
            subnet_size=0,
            port="22",
            notes="allow SSH from any host",
        )
