# Environment: production

terraform {
  required_version = ">= 1.9.0"
  backend "s3" {
    bucket         = "lcc-tfstate-production"
    key            = "terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "lcc-tflock-production"
  }
}

provider "aws" {
  region = "us-east-1"
  default_tags {
    tags = {
      Environment = "production"
      Project     = "lcc"
    }
  }
}

module "postgres" {
  source      = "../../modules/postgres"
  environment = "production"
}

module "redis" {
  source      = "../../modules/redis"
  environment = "production"
}

module "qdrant" {
  source      = "../../modules/qdrant"
  environment = "production"
}

module "vault" {
  source      = "../../modules/vault"
  environment = "production"
}

module "kubernetes_cluster" {
  source      = "../../modules/kubernetes_cluster"
  environment = "production"
}

module "grafana" {
  source      = "../../modules/grafana"
  environment = "production"
}

module "object_storage" {
  source      = "../../modules/object_storage"
  environment = "production"
}

output "postgres_endpoint" { value = module.postgres.endpoint }
output "redis_endpoint"    { value = module.redis.endpoint }
output "qdrant_endpoint"   { value = module.qdrant.endpoint }
output "vault_endpoint"    { value = module.vault.endpoint }
output "grafana_endpoint"  { value = module.grafana.endpoint }
