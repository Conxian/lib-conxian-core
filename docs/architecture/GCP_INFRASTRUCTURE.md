# GCP Infrastructure Documentation

## Strategic Extraction
The Conxian network infrastructure and deployment logic have been fully extracted to the standalone **conxian-gateway** repository. This repository (`lib-conxian-core`) no longer contains infrastructure or deployment artifacts.

### Repository Reference
All GCP infrastructure code, Kubernetes manifests, and deployment pipelines are now managed at:
[https://github.com/Conxian/conxian-gateway](https://github.com/Conxian/conxian-gateway)

### Legacy Path
The previous path `gateway/infrastructure/gcp/` has been removed from this repository to enforce architectural boundaries (CON-700).
