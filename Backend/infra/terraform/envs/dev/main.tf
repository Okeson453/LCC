# Environment: dev

terraform {
  required_version = ">= 1.9.0"
  backend "s3" {
    bucket         = "lcc-tfstate-dev"
    key            = "terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "lcc-tflock-dev"
  }
}

provider "aws" {
  region = "us-east-1"
  default_tags {
    tags = {
      Environment = "dev"
      Project     = "lcc"
    }
  }
}

module "postgres" {
  source      = "../../modules/postgres"
  environment = "dev"
}

module "redis" {
  source      = "../../modules/redis"
  environment = "dev"
}

module "qdrant" {
  source      = "../../modules/qdrant"
  environment = "dev"
}

module "vault" {
  source      = "../../modules/vault"
  environment = "dev"
}

module "kubernetes_cluster" {
  source      = "../../modules/kubernetes_cluster"
  environment = "dev"
}

module "grafana" {
  source      = "../../modules/grafana"
  environment = "dev"
}

module "object_storage" {
  source      = "../../modules/object_storage"
  environment = "dev"
}

output "postgres_endpoint" { value = module.postgres.endpoint }
output "redis_endpoint"    { value = module.redis.endpoint }
output "qdrant_endpoint"   { value = module.qdrant.endpoint }
output "vault_endpoint"    { value = module.vault.endpoint }
output "grafana_endpoint"  { value = module.grafana.endpoint }
