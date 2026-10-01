# Module stub — actual implementation depends on the cloud provider.
# Variables, outputs, and resources are documented in variables.tf and outputs.tf.

variable "environment" { type = string }
variable "name_prefix" { type = string, default = "lcc" }

locals {
  full_name = "${var.name_prefix}-${var.environment}"
}
