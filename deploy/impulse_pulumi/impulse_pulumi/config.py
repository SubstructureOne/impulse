import os

SSH_KEY_PATH = os.getenv("VULTR_SSH_KEY_PATH")
if SSH_KEY_PATH is None:
    raise ValueError("VULTR_SSH_KEY_PATH not set")
