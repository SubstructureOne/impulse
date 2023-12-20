import pulumi
import ediri_vultr as vultr

from .network import KestrelNetwork
from .impulse import ImpulseInstance
from .postgres import ManagedPgInstance, ImpulsePgInstance
from .site import SiteInstance


class Stack:
    def __init__(self, config: pulumi.Config):
        self.network = KestrelNetwork(config)
        self.managed_pg = ManagedPgInstance(config, self.network)
        self.impulse_pg = ImpulsePgInstance(config, self.network)
        self.impulse = ImpulseInstance(config, self.network, self.managed_pg, self.impulse_pg)
        self.site = SiteInstance(config, self.network, self.managed_pg, self.impulse_pg)

        vultr.DnsRecord(
            "impulse_dns",
            vultr.DnsRecordArgs(
                data=self.impulse.instance.main_ip,
                domain="kdeploy.com",
                type="A",
                name=f"impulse.{pulumi.get_stack()}",
            )
        )
        vultr.DnsRecord(
            "kestrel_site_dns",
            vultr.DnsRecordArgs(
                data=self.site.instance.main_ip,
                domain="kdeploy.com",
                type="A",
                name=f"site.{pulumi.get_stack()}",
            )
        )

        pulumi.export("managed_pg_password", self.managed_pg.password.result)
        pulumi.export("managed_pg_publicip", self.managed_pg.instance.main_ip)
        pulumi.export("managed_pg_privateip", self.managed_pg.instance.internal_ip)
        pulumi.export("impulse_pg_password", self.impulse_pg.password.result)
        pulumi.export("impulse_pg_publicip", self.impulse_pg.instance.main_ip)
        pulumi.export("impulse_pg_privateip", self.impulse_pg.instance.internal_ip)
        pulumi.export("impulse_publicip", self.impulse.instance.main_ip)
        pulumi.export("impulse_privateip", self.impulse.instance.internal_ip)
        pulumi.export("site_publicip", self.site.instance.main_ip)
        pulumi.export("site_privateip", self.site.instance.internal_ip)
        pulumi.export("pgpass_key", self.site.pgpass_encryption_key)