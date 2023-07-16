packer {
  required_plugins {
    amazon = {
      version = ">= 1.2.6"
      source = "github.com/hashicorp/amazon"
    }
  }
}

locals {
  timestamp = regex_replace(timestamp(), "[- TZ:]", "")
}

variable "region" {
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
  owners      = ["099720109477"]
  region      = var.region
}

source "amazon-ebs" "impulse" {
  ami_name = "impulse-${local.timestamp}"
  instance_type = "t2.medium"
  region = var.region
  source_ami = "${data.amazon-ami.impulse.id}"
  ssh_username = "ubuntu"

  tags = {
    Name = "impulse-ebs"
    OS = "Ubuntu"
    Release = "22.04"
    Base_AMI_ID = "{{ .SourceAMI }}"
    Base_AMI_Name = "{{ .SourceAMIName }}"
  }

  snapshot_tags = {
    Name= "impulse-ebs-snapshot"
    source = "hashicorp/learn"
    purpose = "demo"
  }
}

build {
  sources = ["source.amazon-ebs.impulse"]

  provisioner "shell" {
    inline = [
      "sudo mkdir -p /opt/impulse/bin",
      "sudo mkdir -p /opt/envoy/bin",
      "sudo mkdir /opt/envoy/etc"
    ]
  }

  provisioner "file" {
    sources = [
      "../target/release/impulse",
      "../target/release/prew",
      "../target/release/create_database"
    ],
    destination = "/opt/impulse/bin/"
  }

  provisioner "file" {
    source = "envoy-postgres.yaml",
    destination = "/opt/envoy/etc/envoy-postgres.yaml"
  }

  provisioner "shell" {
    script           = "./setup.sh"
  }
}
