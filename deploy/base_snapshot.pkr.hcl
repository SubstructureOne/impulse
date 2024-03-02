packer {
  required_plugins {
    vultr = {
      version = ">= 2.5.0"
      source = "github.com/vultr/vultr"
    }
  }
}

locals {
  timestamp = regex_replace(timestamp(), "[- TZ:]", "")
}

variable "vultr_api_key" {
  type = string
}

variable "vultr_region" {
  type = string
}

variable "vultr_ssh_key_id" {
  type = string
}

source "vultr" "impulse" {
  api_key              = "${var.vultr_api_key}"
  os_id                = "1743"  # Ubuntu 22.04 LTS x64
  plan_id              = "vhp-1c-1gb-amd"
  region_id            = "${var.vultr_region}"
  snapshot_description = "base-${local.timestamp}"
  state_timeout        = "10m"
  ssh_username         = "root"
  ssh_key_ids          = ["${var.vultr_ssh_key_id}"]
}

build {
  sources = ["source.vultr.impulse"]

  provisioner "shell" {
    inline = [
      "mkdir -p /setup"
    ]
  }

  provisioner "file" {
    source = "./downloads/fluent-apt-source.deb"
    destination = "/setup/"
  }

  provisioner "shell" {
    script = "./setup_base.sh"
    expect_disconnect = true
    pause_after = "120s"
  }
}
