import pulumi
import ediri_vultr as vultr

from .network import KestrelNetwork
from .impulse import ImpulseInstance
from .postgres import ManagedPgInstance, ImpulsePgInstance
from .site import SiteInstance


class Stack:
    def __init__(self, config: pulumi.Config):
        self.network = KestrelNetwork(config)
        self.impulse_pg = ImpulsePgInstance(config, self.network)
        self.managed_pg = ManagedPgInstance(config, self.network, self.impulse_pg)
        self.impulse = ImpulseInstance(
            config, self.network, self.managed_pg, self.impulse_pg
        )
        self.site = SiteInstance(config, self.network, self.managed_pg, self.impulse_pg)
        self.cluster = vultr.Kubernetes(
            "vke",
            vultr.KubernetesArgs(
                label=f"vke ({pulumi.get_stack()})",
                version=config.require("vke_k8s_version"),
                region=config.require("region"),
                node_pools=vultr.KubernetesNodePoolsArgs(
                    label=f"vk-nodepool ({pulumi.get_stack()})",
                    auto_scaler=True,
                    min_nodes=config.require_int("vke_min_nodes"),
                    max_nodes=config.require_int("vke_max_nodes"),
                    node_quantity=config.require_int("vke_min_nodes"),
                    plan=config.require("vke_plan"),
                ),
            ),
        )
        # self.node_pool = vultr.KubernetesNodePools(
        #     "vke_np",
        #     vultr.KubernetesNodePoolsInitArgs(
        #         cluster_id=self.cluster.id,
        #         label=f"vk-nodepool ({pulumi.get_stack()})",
        #         auto_scaler=True,
        #         min_nodes=config.require_int("vke_min_nodes"),
        #         max_nodes=config.require_int("vke_max_nodes"),
        #         node_quantity=config.require_int("vke_min_nodes"),
        #         plan=config.require("vke_plan"),
        #     ),
        # )

        vultr.DnsRecord(
            "impulse_dns",
            vultr.DnsRecordArgs(
                data=self.impulse.reserved_ip.subnet,
                domain=config.require("impulse_domain"),
                type="A",
                name=config.require("impulse_dnsname"),
            ),
        )
        vultr.DnsRecord(
            "kestrel_site_dns",
            vultr.DnsRecordArgs(
                data=self.site.reserved_ip.subnet,
                domain=config.require("site_domain"),
                type="A",
                name=config.require("site_dnsname"),
            ),
        )

        pulumi.export("managed_pg_password", self.managed_pg.password.result)
        pulumi.export("managed_pg_publicip", self.managed_pg.instance.main_ip)
        pulumi.export("managed_pg_privateip", self.managed_pg.instance.internal_ip)
        pulumi.export("impulse_pg_password", self.impulse_pg.password.result)
        pulumi.export("impulse_pg_publicip", self.impulse_pg.instance.main_ip)
        pulumi.export("impulse_pg_privateip", self.impulse_pg.instance.internal_ip)
        pulumi.export("impulse_publicip", self.impulse.instance.main_ip)
        pulumi.export("impulse_privateip", self.impulse.instance.internal_ip)
        pulumi.export("impulse_staticip", self.impulse.reserved_ip.subnet)
        pulumi.export("site_publicip", self.site.instance.main_ip)
        pulumi.export("site_privateip", self.site.instance.internal_ip)
        pulumi.export("site_staticip", self.site.reserved_ip.subnet)
        pulumi.export("pgpass_key", self.site.pgpass_encryption_key)
