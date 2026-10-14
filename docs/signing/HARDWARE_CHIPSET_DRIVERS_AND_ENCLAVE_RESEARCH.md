# Hardware Chipset Drivers & Enclave Boundary Research (2026 Specification)

This research document specifies the hardware chipset driver requirements, device node interfaces, kernel abstractions, hardware attestation formats, and zero-secret-egress protocol boundaries across all supported Hardware Security Modules (HSMs), Trusted Execution Environments (TEEs), Secure Elements (SEs), and Confidential Computing platforms in the Conxian ecosystem.

---

## 1. Executive Summary & Architectural Guarantees

In the Conxian security architecture:
- **`lib-conxian-core`** defines provider-neutral, secret-safe protocol contracts, hardware chipset enums, attestation evidence structures, and fail-closed structural verifiers. Core never stores, handles, logs, or exports secret key material.
- **`conxius-enclave-sdk`** implements concrete provider driver adapters, attestation certificate path validation against hardware vendor Root CAs (AWS KMS, Google Root CA, Intel PCS, AMD ASK/ARK, Apple App Attest CA), and hardware signing operations.
- **`conxian-gateway`** manages orchestration, transport, and policy enforcement around hardware signing endpoints.
- **`conxius-wallet`** provides client-side hardware binding (StrongBox, Apple SEP, WebAuthn Passkeys, Hardware Wallets).

---

## 2. Comprehensive Hardware Chipset & Driver Matrix

| Hardware Chipset | Vendor / Architecture | Kernel / Device Endpoint | Driver API / Abstraction | Attestation Format | Certificate Root & Measurement |
|---|---|---|---|---|---|
| **AWS Nitro Enclaves** | Annapurna Labs PCIe ASIC | `/dev/nitro_enclave` | NSM (Nitro Secure Module) IOCTLs | COSE Sign1 (CBOR) | AWS KMS Certificate Chain (PCR0–PCR15 measurements) |
| **Android KeyMint / StrongBox** | Titan M2 / Knox SE / TrustZone | `hal:android.hardware.security.keymint` | KeyMint v3 HAL / Binder IPC | X.509 Certificate Chain | Google Root CA (OID `1.3.6.1.4.1.11129.2.1.17` extension) |
| **Intel SGX** | Intel Xeon Scalable (3rd/4th/5th Gen) | `/dev/sgx_enclave`, `/dev/sgx_provision` | Intel DCAP Quoting Library / Enclave Driver | DCAP Quote (ECDSA P-256) | Intel PCS (Provisioning Certification Service) / MRENCLAVE |
| **Intel TDX** | Intel Xeon Scalable (4th/5th Gen) | `/dev/tdx_guest` | Intel TDX Guest Driver / ConfigFS | TDX Quote (ECDSA P-256) | Intel PCS / MRTD (Measurement of Trust Domain) |
| **AMD SEV-SNP** | AMD EPYC (7003/9004 Milan/Genoa) | `/dev/sev-guest` | SEV-SNP Guest IOCTL (`SNP_GET_REPORT`) | SNP Attestation Report | AMD ASK/ARK Certificate Chain / VCEK (SHA-384 launch digest) |
| **Apple Secure Enclave (SEP)** | Apple A-series / M-series SoCs | `sys:apple.framework.localauth` | LocalAuthentication / AppAttest / CryptoTokenKit | Apple DCAppAttest Statement | Apple App Attest Root CA / Hardware-bound key statement |
| **PKCS#11 HSMs** | YubiHSM 2, Thales Luna, AWS CloudHSM | `/usr/lib/libpkcs11.so` (Dynamic PKCS#11 Provider) | Cryptoki C-API (`C_Initialize`, `C_Sign`) | PKCS#11 Slot Attestation | HSM Vendor Certificate / CKA_ALWAYS_SENSITIVE flags |
| **RISC-V Keystone Enclaves** | RISC-V PMP / Security Monitor | `/dev/keystone_enclave` | Keystone Driver / Security Monitor ECALLs | Keystone Attestation Report | SM Public Key / Enclave Measurement Hash & PMP bounds |
| **TPM 2.0** | Discrete / Firmware TPM (Infineon, Nuvoton) | `/dev/tpmrm0` (Resource Manager) | TSS2 (ESYS/FAPI) / TCG Specification | TPM2_Quote over PCRs | TPM Manufacturer EK Cert / AK (Attestation Key) |
| **ARM TrustZone / OP-TEE** | ARM Cortex-A/M TrustZone | `/dev/tee0`, `/dev/teepriv0` | GlobalPlatform TEE Client API / SMC | OP-TEE Attestation Token | TEE Root Key / TA (Trusted Application) Digest |
| **Hardware Wallets & SEs** | Keystone Pro/3, Ledger ST33, Trezor | `/dev/hidraw` / USB HID / NFC / QR / WebUSB | APDU / SLIP-0010 / Transport Protocol | SE Vendor Certificate | STMicroelectronics ST33 SE cert / BIP-32 derivation proof |

---

## 3. Chipset Driver Specifications & Kernel Interfaces

### 3.1 AWS Nitro Enclaves (`AwsNitro`)
- **Device Endpoint**: `/dev/nitro_enclave`
- **Architecture**: Isolated vCPU and memory slices attached via PCIe to Annapurna Labs Nitro Card. No networking, disk storage, or interactive shell access.
- **Communication Channel**: `vsock` (AF_VSOCK port socket communication between parent instance and enclave).
- **Attestation Flow**: Enclave generates internal secp256k1/ed25519 signing keys, requests NSM attestation payload via IOCTL, receiving a COSE Sign1 document signed by the Nitro hypervisor certificate containing PCR0 (EIF image hash), PCR1 (kernel hash), PCR2 (cmdline), PCR3 (IAM role), and PCR8 (enclave image signing certificate).

### 3.2 Android KeyMint / StrongBox (`AndroidStrongBox`)
- **HAL Interface**: `android.hardware.security.keymint.IKeyMintDevice`
- **Hardware Isolation**: Discrete Secure Element (Titan M2 / Knox SE) with tamper-resistant physical package, hardware clock, power glitch detection, and side-channel shielding.
- **Attestation Flow**: Enclave invokes `attestKey()` with a 32-byte challenge nonce. StrongBox returns an X.509 certificate chain rooted in the Google Attestation Root CA containing the Key Attestation Extension (OID `1.3.6.1.4.1.11129.2.1.17`).

### 3.3 Intel SGX & TDX (`IntelSgx`, `IntelTdx`)
- **Device Endpoints**: `/dev/sgx_enclave`, `/dev/tdx_guest`
- **Attestation Flow**: User-space application retrieves quote via Intel DCAP Quoting Library. Architectural Quote Enclave (QE) or TDX Attestation Key Convertor Service (SGX-ECDSA-Attestation) generates a signed quote over `MRENCLAVE`, `MRSIGNER`, `REPORTDATA` (SHA-256 hash of public key and challenge nonce). Verified using Intel PCS PCK certificates.

### 3.4 AMD SEV-SNP (`AmdSevSnp`)
- **Device Endpoint**: `/dev/sev-guest`
- **Attestation Flow**: Guest enclave issues `SNP_GET_REPORT` IOCTL passing a 64-byte user data payload (containing challenge nonce and public key commitment). AMD Platform Security Processor (PSP) constructs SNP Attestation Report signed by Versioned Chip Endorsement Key (VCEK).

### 3.5 Apple Secure Enclave Processor (`AppleSecureEnclave`)
- **Frameworks**: `LocalAuthentication`, `CryptoTokenKit`, `AppAttest`
- **Attestation Flow**: SEP constructs secp256r1 or secp256k1 key pairs flagged with `kSecAccessControlBiometryAny` / `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`. AppAttest yields CBOR attestation statements signed by Apple App Attest CA.

### 3.6 PKCS#11 Hardware Security Modules (`Pkcs11Hsm`)
- **Driver Module**: Shared Object (`/usr/lib/libpkcs11.so`, `yubihsm_pkcs11.so`, etc.)
- **Security Guarantees**: Private key attributes enforced at hardware level:
  - `CKA_PRIVATE = TRUE`
  - `CKA_SENSITIVE = TRUE`
  - `CKA_EXTRACTABLE = FALSE`
  - `CKA_NEVER_EXTRACTABLE = TRUE`
  - `CKA_ALWAYS_SENSITIVE = TRUE`

### 3.7 RISC-V Keystone Enclaves (`RiscVKeystone`)
- **Device Endpoint**: `/dev/keystone_enclave`
- **Driver Abstraction**: Physical Memory Protection (PMP) enforced by Machine-mode Security Monitor (SM). SM generates hardware attestation reports containing SM public key, enclave measurement hash, and user challenge payload.

### 3.8 TPM 2.0 (`Tpm20`)
- **Device Endpoint**: `/dev/tpmrm0`
- **Software Stack**: TSS2 (TPM Software Stack ESYS / FAPI)
- **Attestation Flow**: `TPM2_Quote` operation over configured PCR selection signed by Attestation Key (AK), bound to Endorsement Key (EK) certificate issued by TPM manufacturer.

### 3.9 ARM TrustZone / OP-TEE (`ArmTrustZone`)
- **Device Endpoint**: `/dev/tee0`, `/dev/teepriv0`
- **Driver Abstraction**: Linux OP-TEE kernel driver issuing Secure Monitor Calls (SMC). Trusted Application (TA) generates attestation tokens signed by TrustZone Root Key.

### 3.10 Hardware Signers & Secure Elements (`HardwareWallet`)
- **Transport Interfaces**: USB HID (`/dev/hidraw`), NFC, WebUSB, Serial, QR
- **Security Element**: STMicroelectronics ST33 or Microchip ATECC608A SE chip executing isolated BIP-32 key derivation and Taproot/Schnorr signature creation.

---

## 4. Fail-Closed Protocol Integration in `lib-conxian-core`

`lib-conxian-core` enforces strict structural validation on hardware attestation evidence:
1. **`HardwareChipset`**: Enums represent all 11 target hardware environments.
2. **`AttestationType`**: Enums specify exact attestation payload structures.
3. **Chipset-Attestation Pairing**: `attestation_type.is_compatible_with(chipset)` must return `true`.
4. **Challenge Nonce Enforcement**: Must be exactly 32 non-zero bytes.
5. **Measurement Digest Enforcement**: Must be at least 32 bytes (SHA-256 / SHA-384 / SHA-512 / SM3).
6. **Device Node Validation**: Driver info device node paths must use canonical system prefixes (`/dev/`, `hal:`, `sys:`, `/usr/lib`).
7. **Fail-Closed Result**: `HeadlessEnclave::verify_hardware_attestation_evidence` performs structural verification and returns `Err(EnclaveVerificationError::UnsupportedProvider)`, guaranteeing that zero software mock signing or unverified hardware attestation can satisfy value-bearing operations in Core.

---

## 5. Cross-Repository Alignment Matrix

| Repository | Responsibility | Hardware Signing Artifact |
|---|---|---|
| **`lib-conxian-core`** | Zero-secret-egress protocol contracts & structural verification | `HardwareChipset`, `AttestationType`, `HardwareAttestationEvidence`, `ChipsetDriverInfo` |
| **`conxius-enclave-sdk`** | Driver adapters & cryptographic attestation verification | `EnclaveManager`, Nitro/StrongBox/PKCS11/WebAuthn driver implementations |
| **`conxian-gateway`** | Runtime service orchestration & RPC/enclave connectivity | `conxian verify` pre-flight checks, KMS/enclave endpoints |
| **`conxius-wallet`** | Client hardware key custody & passkey authorization | Mobile StrongBox HAL, SEP, WebAuthn, Hardware Wallet drivers |
