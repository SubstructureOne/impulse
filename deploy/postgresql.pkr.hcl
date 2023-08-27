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

variable "region" {
  type = string
}

variable "postgres_password" {
  type = string
}

variable "postgres_purpose" {
  type = string
}

variable "vultr_api_key" {
  type = string
}

variable "vultr_plan_id" {
  type = string
}

variable "vultr_region" {
  type = string
}

variable "snapshot_name" {
  type = string
}

source "vultr" "impulse" {
  api_key              = "${var.vultr_api_key}"
  os_id                = "1743"  # Ubuntu 22.04 LTS x64
  plan_id              = "${var.vultr_plan_id}"
  region_id            = "${var.vultr_region}"
  snapshot_description = "${var.snapshot_name}-${local.timestamp}"
  state_timeout        = "10m"
  ssh_username         = "root"
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
    script = "./setup_pg.sh"
    environment_vars = [
      "POSTGRES_PASSWORD=${var.postgres_password}",
      "POSTGRES_PURPOSE=${var.postgres_purpose}"
    ]
  }
}
