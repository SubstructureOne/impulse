import pulumi

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
        self.site = SiteInstance(config, self.network, self.impulse_pg)
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