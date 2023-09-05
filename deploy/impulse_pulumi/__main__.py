"""Pulumi configuration for deploying Impulse"""
import pulumi

from impulse_pulumi.stack import Stack

config = pulumi.Config()
stack = Stack(config)
