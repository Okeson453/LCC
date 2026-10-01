variable "environment" {
  type        = string
  description = "Deployment environment (local, dev, staging, production)"
}

variable "name_prefix" {
  type        = string
  description = "Prefix for resource names"
  default     = "lcc"
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to resources"
  default     = {}
}
