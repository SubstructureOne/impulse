packer {
  required_plugins {
    amazon = {
      version = ">= 1.2.6"
      source = "github.com/hashicorp/amazon"
    }
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

variable "vultr_api_key" {
  type = string
}

variable "vultr_plan_id" {
  type = string
}

variable "vultr_region" {
  type = string
}

data "amazon-ami" "impulse" {
  filters = {
    architecture = "x86_64"
    "block-device-mapping.volume-type" = "gp2"
    name = "ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"
    root-device-type = "ebs"
    virtualization-type = "hvm"
  }
  most_recent = true
  owners = ["099720109477"]
  region = var.region
}

source "amazon-ebs" "impulse" {
  ami_name = "impulse-${local.timestamp}"
  instance_type = "t2.micro"
  region = var.region
  source_ami = "${data.amazon-ami.impulse.id}"
  ssh_username = "ubuntu"

  tags = {
    Name = "impulse-ami"
    OS = "Ubuntu"
    Release = "22.04"
    Base_AMI_ID = "{{ .SourceAMI }}"
    Base_AMI_Name = "{{ .SourceAMIName }}"
  }

  snapshot_tags = {
    Name = "impulse-ami-snapshot"
    source = "hashicorp/learn"
    purpose = "demo"
  }
}

source "vultr" "impulse" {
  api_key              = "${var.vultr_api_key}"
  os_id                = "1743"  # Ubuntu 22.04 LTS x64
  plan_id              = "${var.vultr_plan_id}"
  region_id            = "${var.vultr_region}"
  snapshot_description = "impulse-dev"
  state_timeout        = "10m"
  ssh_username         = "root"
}

build {
#  sources = ["source.amazon-ebs.impulse"]
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
    script = "./setup.sh"
    environment_vars = [
      "POSTGRES_PASSWORD=${var.postgres_password}"
    ]
  }
}
