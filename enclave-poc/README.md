# Conxian AWS Nitro Enclave Signing — Proof of Concept & Deployment Guide

## Overview

This POC demonstrates the complete enclave signing pipeline for `lib-conxian-core` using AWS Nitro Enclaves. The project proves that:

1. **Core types flow correctly** through the enclave adapter (`lib-conxian-core-enclave`)
2. **Trust policy gates work** — Strict/Managed/Expedient/ObserverOnly tiers enforced
3. **Attestation evidence** is generated and validated
4. **Fail-closed behavior** is maintained at every boundary
5. **The signing pipeline is production-ready** when backed by a real Nitro Enclave

## Quick Start (POC)

```bash
# Build and run the Rust POC binary
cd enclave-poc
cargo run

# This demonstrates 4 scenarios:
# 1. Strict-tier Bitcoin signing with Nitro TEE ✅
# 2. ObserverOnly tier — signing REJECTED ✅
# 3. Strict tier with software attestation — REJECTED ✅
# 4. Full chain signing flow (Bitcoin, Ethereum, Solana, Stacks, Babylon)
```

## Test Results Summary

| Layer | Tests | Result |
|-------|-------|--------|
| Core lib tests | 109 | ✅ All pass |
| Integration tests | ~90 | ✅ All pass |
| Adapter tests | 28 | ✅ All pass |
| Doc tests | 5/7 | ✅ Pass (2 intentionally ignored) |
| **Total** | **227+** | **0 failures** |

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  EC2 Parent Instance (m5.xlarge+, Nitro-enabled)                  │
│                                                                   │
│  ┌─────────────────────┐     ┌──────────────────────────────────┐ │
│  │ lib-conxian-core    │     │ AWS Nitro Enclave                 │ │
│  │                     │     │                                   │ │
│  │ 1. SignRequest      │vsock│ 2. Validate request               │ │
│  │    (Core types)     │────▶│ 3. Sign with enclave key          │ │
│  │                     │     │ 4. Generate attestation doc        │ │
│  │ 6. EnclaveSignResp  │◀────│ 5. Return sig + attestation       │ │
│  │    + Attestation    │     │                                   │ │
│  │                     │     │ PCR measurements:                 │ │
│  │ 7. Validate         │     │  PCR0: Enclave image hash          │ │
│  │    attestation       │     │  PCR1: Linux kernel hash           │ │
│  │    evidence          │     │  PCR2: Application hash            │ │
│  │                     │     │  PCR3: IAM role hash                │ │
│  │ 8. Map to Core       │     │                                   │ │
│  │    SignResponse      │     │                                   │ │
│  └─────────────────────┘     └──────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

## AWS Nitro Enclave Production Deployment

### Prerequisites

1. **EC2 Instance:** Any Nitro-enabled instance type:
   - `m5.xlarge`+ (general purpose)
   - `c5.xlarge`+ (compute optimized)
   - `r5.xlarge`+ (memory optimized)
   - Must be launched with `EnclaveOptions.Enabled=true`

2. **IAM Role:** Instance profile with:
   ```json
   {
     "Effect": "Allow",
     "Action": [
       "kms:Decrypt",
       "kms:GenerateDataKey"
     ],
     "Resource": "arn:aws:kms:region:account:key/enclave-signing-key"
   }
   ```

3. **Security Group:** Restrict to vsock only (no network access for enclave)

### Deployment Steps

#### 1. Launch Nitro-enabled EC2 Instance

```bash
aws ec2 run-instances \
    --instance-type m5.xlarge \
    --image-id ami-0c55b159cbfafe1f0 \
    --enclave-options Enabled=true \
    --iam-instance-profile Name=NitroEnclaveSigningRole \
    --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=conxian-nitro-signer}]'
```

#### 2. Install Nitro Enclaves CLI on Parent Instance

```bash
sudo amazon-linux-extras install aws-nitro-enclaves-cli -y
sudo yum install aws-nitro-enclaves-cli-devel -y

# Enable and start the Nitro Enclaves allocator
sudo systemctl enable nitro-enclaves-allocator.service
sudo systemctl start nitro-enclaves-allocator.service

# Verify
nitro-cli --version
```

#### 3. Configure Enclave Resources

Edit `/etc/nitro_enclaves/allocator.yaml`:
```yaml
memory_mib: 512
cpu_count: 2
```

```bash
sudo systemctl restart nitro-enclaves-allocator.service
```

#### 4. Build the Enclave Image (EIF)

```bash
# Build the Docker image
cd enclave-poc
docker build -t conxian-enclave:latest -f docker/Dockerfile.enclave .

# Convert to Enclave Image File
nitro-cli build-enclave \
    --docker-uri conxian-enclave:latest \
    --output-file conxian-enclave.eif

# Record PCR measurements for attestation verification
# PCR0 = SHA384 of the enclave image (deterministic)
```

#### 5. Run the Enclave

```bash
# Start the enclave
nitro-cli run-enclave \
    --eif-path conxian-enclave.eif \
    --memory 512 \
    --cpu-count 2 \
    --enclave-cid 4

# Verify enclave is running
nitro-cli describe-enclaves
```

#### 6. Deploy the Signing Service on Parent

```bash
# Build the Rust signing service
cd enclave-poc
cargo build --release

# Run the service (connects to enclave via vsock CID 4)
./target/release/enclave-poc
```

### Attestation Verification Flow

```
1. Parent sends SignRequest → Enclave via vsock
2. Enclave signs with NSM-derived key
3. Enclave calls /dev/nsm to get attestation document
4. Attestation includes:
   - PCR0-PCR3 measurements (cryptographic proof of enclave identity)
   - Nonce (from request, prevents replay)
   - Public key (NSM-derived, unique to this enclave instance)
5. Parent receives signature + attestation
6. Parent verifies:
   a. Attestation document signature (via AWS KMS or Nitro root CA)
   b. PCR values match expected measurements
   c. Nonce matches the request
7. Response is mapped to Core SignResponse types
```

### Security Properties

| Property | Mechanism |
|----------|-----------|
| **Code integrity** | PCR0 = SHA384 of EIF (immutable once built) |
| **Runtime integrity** | PCR1-3 = kernel + app + IAM role (verified at boot) |
| **Key isolation** | NSM-derived keys never leave the enclave |
| **Replay protection** | Nonce binding in attestation document |
| **Trust policy** | Core adapter enforces Strict/Managed/Expedient gates |
| **Fail-closed** | Any attestation mismatch → typed error, no signature |

### Cost Estimate

| Resource | Spec | Monthly (est.) |
|----------|------|---------------|
| EC2 m5.xlarge | 4 vCPU, 16 GB | ~$140 (on-demand) |
| Nitro Enclave | Included with EC2 | $0 |
| KMS key | 1 key | ~$1 |
| **Total** | | **~$141/month** |

### Production Readiness Checklist

- [ ] POC binary tests pass (227+ tests, 0 failures)
- [ ] Docker image builds for enclave
- [ ] EIF generation with deterministic PCR0
- [ ] Nitro-enabled EC2 instance provisioned
- [ ] Enclave runs and responds on vsock
- [ ] Attestation verification against Nitro root CA
- [ ] Trust policy gates validated end-to-end
- [ ] CI/CD pipeline for EIF builds
- [ ] Monitoring and alerting for enclave health
- [ ] Key rotation procedures documented
