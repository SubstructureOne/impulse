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

variable "base_snapshot_id" {
  type = string
}

variable "ssh_key_file" {
  type = string
}

source "vultr" "impulse" {
  api_key              = "${var.vultr_api_key}"
  snapshot_id          = "${var.base_snapshot_id}"
  plan_id              = "vhp-1c-1gb-amd"
  region_id            = "${var.vultr_region}"
  snapshot_description = "impulse-${local.timestamp}"
  state_timeout        = "10m"
  ssh_username         = "root"
  ssh_private_key_file = "${var.ssh_key_file}"
}

build {
  sources = ["source.vultr.impulse"]

  provisioner "shell" {
    inline = [
      "sudo mkdir -p /setup/release",
      "sudo chmod -R 777 /setup"
    ]
  }

  provisioner "file" {
    sources = [
      "../target/release/impulse",
      "../target/release/prew",
      "../target/release/setup_database"
    ]
    destination = "/setup/release/"
  }

  provisioner "file" {
    sources = [
      "../migrations"
    ]
    destination = "/setup/"
  }

  provisioner "file" {
    source = "image_files"
    destination = "/setup/"
  }

  provisioner "shell" {
    script = "./setup_impulse.sh"
  }
}
