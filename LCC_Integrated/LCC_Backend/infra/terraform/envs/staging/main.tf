# Environment: staging

terraform {
  required_version = ">= 1.9.0"
  backend "s3" {
    bucket         = "lcc-tfstate-staging"
    key            = "terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "lcc-tflock-staging"
  }
}

provider "aws" {
  region = "us-east-1"
  default_tags {
    tags = {
      Environment = "staging"
      Project     = "lcc"
    }
  }
}

module "postgres" {
  source      = "../../modules/postgres"
  environment = "staging"
}

module "redis" {
  source      = "../../modules/redis"
  environment = "staging"
}

module "qdrant" {
  source      = "../../modules/qdrant"
  environment = "staging"
}

module "vault" {
  source      = "../../modules/vault"
  environment = "staging"
}

module "kubernetes_cluster" {
  source      = "../../modules/kubernetes_cluster"
  environment = "staging"
}

module "grafana" {
  source      = "../../modules/grafana"
  environment = "staging"
}

module "object_storage" {
  source      = "../../modules/object_storage"
  environment = "staging"
}

output "postgres_endpoint" { value = module.postgres.endpoint }
output "redis_endpoint"    { value = module.redis.endpoint }
output "qdrant_endpoint"   { value = module.qdrant.endpoint }
output "vault_endpoint"    { value = module.vault.endpoint }
output "grafana_endpoint"  { value = module.grafana.endpoint }
