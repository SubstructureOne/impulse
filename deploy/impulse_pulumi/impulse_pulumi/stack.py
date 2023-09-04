import pulumi

from .network import KestrelNetwork
from .impulse import ImpulseInstance
from .postgres import ManagedPgInstance, ImpulsePgInstance


class Stack:
    def __init__(self, config: pulumi.Config):
        self.network = KestrelNetwork(config)
        self.managed_pg = ManagedPgInstance(config, self.network)
        self.impulse_pg = ImpulsePgInstance(config, self.network)
        self.impulse = ImpulseInstance(config, self.network, self.managed_pg, self.impulse_pg)
