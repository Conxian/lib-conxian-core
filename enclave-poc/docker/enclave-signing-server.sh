#!/bin/sh
# ─────────────────────────────────────────────────────────────────────────────
# AWS Nitro Enclave Signing Server (POC)
#
# Runs inside the Nitro Enclave. Listens on vsock port 5000 for signing
# requests from the parent instance. No network access available.
#
# Protocol (newline-delimited JSON over vsock):
#   REQUEST:  {"algorithm":"SchnorrSecp256k1","message_hash":"...","derivation_path":"..."}
#   RESPONSE: {"signature":"...","public_key":"...","attestation":"...","status":"ok"}
#
# In production, this is the conxius-enclave-sdk Nitro binary with:
#   - Secure key derivation from NSM (Nitro Security Module)
#   - Hardware-backed attestation document generation
#   - PCR measurement verification
# ─────────────────────────────────────────────────────────────────────────────

PORT=5000
CID=3  # Default parent CID for vsock

echo "[ENCLAVE] Starting Nitro Enclave signing server..."
echo "[ENCLAVE] Listening on vsock port $PORT"

# In a real enclave, we'd use socat with VSOCK:
#   socat VSOCK-LISTEN:$PORT,fork EXEC:/usr/local/bin/sign-handler
#
# For this POC, we simulate with a simple loop
while true; do
    # Read a signing request
    read -r REQUEST
    
    if [ -z "$REQUEST" ]; then
        continue
    fi
    
    echo "[ENCLAVE] Received: $REQUEST"
    
    # Extract fields (simplified — real impl uses jq or proper JSON parser)
    ALGO=$(echo "$REQUEST" | grep -o '"algorithm":"[^"]*"' | cut -d'"' -f4)
    MSG_HASH=$(echo "$REQUEST" | grep -o '"message_hash":"[^"]*"' | cut -d'"' -f4)
    
    # Generate deterministic mock signature (real impl uses NSM-derived key)
    SIG="nitro_signature_$(echo "$MSG_HASH" | sha256sum | cut -d' ' -f1)"
    
    # In production, generate real attestation via /dev/nsm
    # For POC, generate a mock attestation document
    ATTESTATION=$(cat <<EOF
{
  "module_id": "i-0a1b2c3d4e5f-enc-0a1b2c3d4e5f",
  "timestamp": $(date +%s),
  "digest": "SHA384",
  "pcrs": {
    "PCR0": "enclave_image_hash_$(sha256sum /usr/local/bin/enclave-signing-server | cut -d' ' -f1)",
    "PCR1": "linux_kernel_hash",
    "PCR2": "application_hash",
    "PCR3": "iam_role_hash"
  },
  "certificate": "NITRO_ATTESTATION_CERT_PLACEHOLDER",
  "user_data": "$MSG_HASH",
  "nonce": "$MSG_HASH"
}
EOF
)
    
    # Send response
    RESPONSE=$(cat <<EOF
{"signature":"$SIG","public_key":"enclave_pubkey_hex","attestation":$ATTESTATION,"algorithm":"$ALGO","status":"ok"}
EOF
)
    
    echo "$RESPONSE"
    echo "[ENCLAVE] Response sent"
done
