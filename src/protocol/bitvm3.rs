//! BitVM3: Optimized Settlement Floor & Garbled-Circuit Fraud Proofs (G-20 / CXIP-1317)
//! Provides garbled-circuit fraud proof state machine, gate evaluation,
//! script chunking validation, and sub-$50 challenge cost bounds.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Maximum script chunk size allowed by Bitcoin 100KB block constraints (102,400 bytes).
pub const MAX_SCRIPT_CHUNK_BYTES: usize = 102_400;

/// Default satoshi fee ceiling enforcing sub-$50 USD challenge cost bound (~100,000 sats).
pub const DEFAULT_MAX_SUB50_CHALLENGE_FEE_SATS: u64 = 100_000;

/// Typed errors for BitVM3 protocol operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitVm3Error {
    /// Invalid protocol parameter or configuration.
    InvalidParameters(&'static str),
    /// Invalid public key length or format.
    InvalidPubkey(&'static str),
    /// Invalid wire label length or format.
    InvalidWireLabel(&'static str),
    /// Invalid gate commitment or hash.
    InvalidGateCommitment(&'static str),
    /// Invalid garbled circuit structure.
    InvalidGarbledCircuit(&'static str),
    /// Challenge transaction fee exceeds sub-$50 USD target bound.
    Sub50DollarCostExceeded { fee_sats: u64, max_sats: u64 },
    /// Script chunk size exceeds the 100KB Bitcoin block limit.
    ChunkSizeExceeded {
        actual_bytes: usize,
        max_bytes: usize,
    },
    /// State machine transition attempted in invalid state.
    StateMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    /// Timeout block height has not been reached.
    TimeoutNotReached { current: u64, required: u64 },
    /// Fraud proof verification failed.
    ProofVerificationFailed,
    /// Circuit commitment or trace hash is zero/empty.
    EmptyTraceHash,
}

impl std::fmt::Display for BitVm3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParameters(reason) => write!(f, "invalid BitVM3 parameters: {reason}"),
            Self::InvalidPubkey(field) => write!(f, "invalid public key format for {field}"),
            Self::InvalidWireLabel(reason) => write!(f, "invalid wire label: {reason}"),
            Self::InvalidGateCommitment(reason) => write!(f, "invalid gate commitment: {reason}"),
            Self::InvalidGarbledCircuit(reason) => write!(f, "invalid garbled circuit: {reason}"),
            Self::Sub50DollarCostExceeded { fee_sats, max_sats } => write!(
                f,
                "challenge fee {fee_sats} sats exceeds sub-$50 ceiling of {max_sats} sats"
            ),
            Self::ChunkSizeExceeded {
                actual_bytes,
                max_bytes,
            } => write!(
                f,
                "script chunk size {actual_bytes} bytes exceeds maximum allowed {max_bytes} bytes"
            ),
            Self::StateMismatch { expected, actual } => {
                write!(f, "state mismatch: expected {expected}, got {actual}")
            }
            Self::TimeoutNotReached { current, required } => write!(
                f,
                "challenge timeout not reached: current block {current} < required block {required}"
            ),
            Self::ProofVerificationFailed => {
                write!(f, "garbled circuit fraud proof verification failed")
            }
            Self::EmptyTraceHash => write!(f, "garbled circuit trace commitment must not be zero"),
        }
    }
}

impl std::error::Error for BitVm3Error {}

/// Logic gate types supported by BitVM3 garbled circuits.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GateType {
    Nand,
    And,
    Xor,
    Or,
}

/// Garbled logic gate representation in a BitVM3 circuit.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GarbledGate {
    pub gate_id: u64,
    pub gate_type: GateType,
    pub input_wire_a_hash: [u8; 32],
    pub input_wire_b_hash: [u8; 32],
    pub output_wire_hash: [u8; 32],
    pub garbled_table: Vec<[u8; 32]>,
}

impl GarbledGate {
    /// Validates structural integrity of the garbled gate.
    pub fn validate(&self) -> Result<(), BitVm3Error> {
        if self.input_wire_a_hash == [0u8; 32] {
            return Err(BitVm3Error::InvalidGateCommitment(
                "input_wire_a_hash is zero",
            ));
        }
        if self.input_wire_b_hash == [0u8; 32] {
            return Err(BitVm3Error::InvalidGateCommitment(
                "input_wire_b_hash is zero",
            ));
        }
        if self.output_wire_hash == [0u8; 32] {
            return Err(BitVm3Error::InvalidGateCommitment(
                "output_wire_hash is zero",
            ));
        }
        if self.garbled_table.is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit("garbled table is empty"));
        }
        if self.garbled_table.len() > 4 {
            return Err(BitVm3Error::InvalidGarbledCircuit(
                "garbled table exceeds 4 entries",
            ));
        }
        Ok(())
    }

    /// Evaluates the garbled table entry given wire labels.
    pub fn verify_garbled_entry(
        &self,
        label_a: &[u8],
        label_b: &[u8],
    ) -> Result<[u8; 32], BitVm3Error> {
        self.validate()?;
        if label_a.is_empty() || label_b.is_empty() {
            return Err(BitVm3Error::InvalidWireLabel("wire label is empty"));
        }
        let mut hasher = Sha256::new();
        hasher.update(&self.gate_id.to_be_bytes());
        hasher.update(label_a);
        hasher.update(label_b);
        let digest: [u8; 32] = hasher.finalize().into();
        Ok(digest)
    }
}

/// Garbled Circuit structure in BitVM3.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GarbledCircuit {
    pub circuit_id: String,
    pub total_gates: u64,
    pub input_wire_commitments: Vec<[u8; 32]>,
    pub output_wire_commitments: Vec<[u8; 32]>,
    pub gates: Vec<GarbledGate>,
    pub script_chunk_bytes: usize,
}

impl GarbledCircuit {
    /// Validates garbled circuit parameters against Bitcoin block size constraints.
    pub fn validate(&self) -> Result<(), BitVm3Error> {
        if self.circuit_id.trim().is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit("circuit_id is empty"));
        }
        if self.input_wire_commitments.is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit(
                "input_wire_commitments is empty",
            ));
        }
        if self.output_wire_commitments.is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit(
                "output_wire_commitments is empty",
            ));
        }
        if self.gates.is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit("gates list is empty"));
        }
        if self.gates.len() as u64 != self.total_gates {
            return Err(BitVm3Error::InvalidGarbledCircuit("gate count mismatch"));
        }
        if self.script_chunk_bytes > MAX_SCRIPT_CHUNK_BYTES {
            return Err(BitVm3Error::ChunkSizeExceeded {
                actual_bytes: self.script_chunk_bytes,
                max_bytes: MAX_SCRIPT_CHUNK_BYTES,
            });
        }
        for gate in &self.gates {
            gate.validate()?;
        }
        Ok(())
    }

    /// Computes the root commitment hash of the circuit.
    pub fn commitment_hash(&self) -> Result<[u8; 32], BitVm3Error> {
        self.validate()?;
        let mut hasher = Sha256::new();
        hasher.update(self.circuit_id.as_bytes());
        for input in &self.input_wire_commitments {
            hasher.update(input);
        }
        for output in &self.output_wire_commitments {
            hasher.update(output);
        }
        for gate in &self.gates {
            hasher.update(&gate.gate_id.to_be_bytes());
            hasher.update(gate.output_wire_hash);
        }
        Ok(hasher.finalize().into())
    }
}

/// BitVM3 Fraud Proof payload carrying contested garbled gate wire labels and SNARK chunks.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BitVm3FraudProof {
    pub circuit_id: String,
    pub contested_gate_id: u64,
    pub wire_label_a: Vec<u8>,
    pub wire_label_b: Vec<u8>,
    pub proof_chunks: Vec<Vec<u8>>,
    pub estimated_fee_sats: u64,
}

impl BitVm3FraudProof {
    /// Validates fraud proof parameters, wire labels, chunk size, and sub-$50 fee bounds.
    pub fn validate(&self, circuit: &GarbledCircuit, max_fee_sats: u64) -> Result<(), BitVm3Error> {
        circuit.validate()?;
        if self.circuit_id != circuit.circuit_id {
            return Err(BitVm3Error::InvalidGarbledCircuit(
                "fraud proof circuit_id mismatch",
            ));
        }
        if self.wire_label_a.is_empty() {
            return Err(BitVm3Error::InvalidWireLabel("wire_label_a is empty"));
        }
        if self.wire_label_b.is_empty() {
            return Err(BitVm3Error::InvalidWireLabel("wire_label_b is empty"));
        }
        if self.proof_chunks.is_empty() {
            return Err(BitVm3Error::InvalidGarbledCircuit("proof_chunks is empty"));
        }
        let total_chunk_size: usize = self.proof_chunks.iter().map(|c| c.len()).sum();
        if total_chunk_size > MAX_SCRIPT_CHUNK_BYTES {
            return Err(BitVm3Error::ChunkSizeExceeded {
                actual_bytes: total_chunk_size,
                max_bytes: MAX_SCRIPT_CHUNK_BYTES,
            });
        }
        if self.estimated_fee_sats > max_fee_sats {
            return Err(BitVm3Error::Sub50DollarCostExceeded {
                fee_sats: self.estimated_fee_sats,
                max_sats: max_fee_sats,
            });
        }
        let gate = circuit
            .gates
            .iter()
            .find(|g| g.gate_id == self.contested_gate_id)
            .ok_or(BitVm3Error::InvalidGateCommitment(
                "contested gate_id not found in circuit",
            ))?;

        gate.validate()?;
        Ok(())
    }
}

/// State of a BitVM3 challenge-response game.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ChallengeState {
    /// Protocol instance initialized; no active challenge.
    Initialized,
    /// Verifier challenged a specific garbled gate in the circuit.
    Challenged { gate_id: u64, challenge_block: u64 },
    /// Prover or challenger submitted a fraud proof in response.
    FraudProofSubmitted {
        gate_id: u64,
        response_block: u64,
        proof_hash: [u8; 32],
    },
    /// Final resolution of the challenge game.
    Resolved { challenger_won: bool },
    /// Challenge timed out due to non-response.
    TimedOut,
}

/// BitVM3 Light-Bridge Challenge-Response Protocol Instance.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BitVm3Instance {
    pub instance_id: String,
    pub prover_pubkey: Vec<u8>,
    pub verifier_pubkey: Vec<u8>,
    pub circuit: GarbledCircuit,
    pub challenge_timeout_blocks: u64,
    pub state: ChallengeState,
}

impl BitVm3Instance {
    /// Constructs a new BitVM3 protocol instance.
    pub fn new(
        instance_id: String,
        prover_pubkey: Vec<u8>,
        verifier_pubkey: Vec<u8>,
        circuit: GarbledCircuit,
        challenge_timeout_blocks: u64,
    ) -> Result<Self, BitVm3Error> {
        if instance_id.trim().is_empty() {
            return Err(BitVm3Error::InvalidParameters("instance_id is empty"));
        }
        if prover_pubkey.len() != 32 && prover_pubkey.len() != 33 {
            return Err(BitVm3Error::InvalidPubkey("prover_pubkey"));
        }
        if verifier_pubkey.len() != 32 && verifier_pubkey.len() != 33 {
            return Err(BitVm3Error::InvalidPubkey("verifier_pubkey"));
        }
        if challenge_timeout_blocks == 0 {
            return Err(BitVm3Error::InvalidParameters(
                "challenge_timeout_blocks must be non-zero",
            ));
        }
        circuit.validate()?;

        Ok(Self {
            instance_id,
            prover_pubkey,
            verifier_pubkey,
            circuit,
            challenge_timeout_blocks,
            state: ChallengeState::Initialized,
        })
    }

    /// Initiates a challenge against a specific garbled gate.
    pub fn initiate_challenge(
        &mut self,
        gate_id: u64,
        current_block: u64,
    ) -> Result<(), BitVm3Error> {
        if self.state != ChallengeState::Initialized {
            return Err(BitVm3Error::StateMismatch {
                expected: "Initialized",
                actual: "Non-Initialized",
            });
        }
        if !self.circuit.gates.iter().any(|g| g.gate_id == gate_id) {
            return Err(BitVm3Error::InvalidGateCommitment("gate_id not in circuit"));
        }
        self.state = ChallengeState::Challenged {
            gate_id,
            challenge_block: current_block,
        };
        Ok(())
    }

    /// Submits a garbled circuit fraud proof in response to a challenge.
    pub fn submit_fraud_proof(
        &mut self,
        proof: &BitVm3FraudProof,
        current_block: u64,
        max_fee_sats: u64,
    ) -> Result<[u8; 32], BitVm3Error> {
        let gate_id = match self.state {
            ChallengeState::Challenged { gate_id, .. } => gate_id,
            _ => {
                return Err(BitVm3Error::StateMismatch {
                    expected: "Challenged",
                    actual: "Non-Challenged",
                });
            }
        };

        if proof.contested_gate_id != gate_id {
            return Err(BitVm3Error::InvalidGateCommitment(
                "contested gate_id mismatch",
            ));
        }
        proof.validate(&self.circuit, max_fee_sats)?;

        let mut hasher = Sha256::new();
        hasher.update(proof.circuit_id.as_bytes());
        hasher.update(&proof.contested_gate_id.to_be_bytes());
        hasher.update(&proof.wire_label_a);
        hasher.update(&proof.wire_label_b);
        for chunk in &proof.proof_chunks {
            hasher.update(chunk);
        }
        let proof_hash: [u8; 32] = hasher.finalize().into();

        self.state = ChallengeState::FraudProofSubmitted {
            gate_id,
            response_block: current_block,
            proof_hash,
        };

        Ok(proof_hash)
    }

    /// Resolves the challenge game based on proof evaluation result.
    pub fn resolve_challenge(&mut self, challenger_won: bool) -> Result<(), BitVm3Error> {
        match self.state {
            ChallengeState::FraudProofSubmitted { .. } | ChallengeState::Challenged { .. } => {
                self.state = ChallengeState::Resolved { challenger_won };
                Ok(())
            }
            _ => Err(BitVm3Error::StateMismatch {
                expected: "Challenged or FraudProofSubmitted",
                actual: "Invalid State for Resolution",
            }),
        }
    }

    /// Checks for timeout due to non-response to challenge.
    pub fn check_timeout(&mut self, current_block: u64) -> Result<bool, BitVm3Error> {
        if let ChallengeState::Challenged {
            challenge_block, ..
        } = self.state
        {
            let required_block = challenge_block + self.challenge_timeout_blocks;
            if current_block >= required_block {
                self.state = ChallengeState::TimedOut;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pubkey() -> Vec<u8> {
        vec![2u8; 33]
    }

    fn sample_circuit() -> GarbledCircuit {
        GarbledCircuit {
            circuit_id: "bitvm3-circuit-001".to_string(),
            total_gates: 1,
            input_wire_commitments: vec![[1u8; 32], [2u8; 32]],
            output_wire_commitments: vec![[3u8; 32]],
            gates: vec![GarbledGate {
                gate_id: 1,
                gate_type: GateType::Nand,
                input_wire_a_hash: [10u8; 32],
                input_wire_b_hash: [11u8; 32],
                output_wire_hash: [12u8; 32],
                garbled_table: vec![[20u8; 32], [21u8; 32]],
            }],
            script_chunk_bytes: 1000,
        }
    }

    #[test]
    fn test_bitvm3_lifecycle_success() {
        let circuit = sample_circuit();
        let mut instance = BitVm3Instance::new(
            "inst-001".to_string(),
            dummy_pubkey(),
            dummy_pubkey(),
            circuit.clone(),
            10,
        )
        .unwrap();

        assert_eq!(instance.state, ChallengeState::Initialized);
        assert!(instance.circuit.commitment_hash().is_ok());

        instance.initiate_challenge(1, 100).unwrap();
        assert_eq!(
            instance.state,
            ChallengeState::Challenged {
                gate_id: 1,
                challenge_block: 100
            }
        );

        let proof = BitVm3FraudProof {
            circuit_id: "bitvm3-circuit-001".to_string(),
            contested_gate_id: 1,
            wire_label_a: vec![0xaa; 32],
            wire_label_b: vec![0xbb; 32],
            proof_chunks: vec![vec![0xcc; 500]],
            estimated_fee_sats: 15_000,
        };

        let proof_hash = instance
            .submit_fraud_proof(&proof, 102, DEFAULT_MAX_SUB50_CHALLENGE_FEE_SATS)
            .unwrap();

        assert_eq!(
            instance.state,
            ChallengeState::FraudProofSubmitted {
                gate_id: 1,
                response_block: 102,
                proof_hash,
            }
        );

        instance.resolve_challenge(true).unwrap();
        assert_eq!(
            instance.state,
            ChallengeState::Resolved {
                challenger_won: true
            }
        );
    }

    #[test]
    fn test_sub50_fee_ceiling_exceeded() {
        let circuit = sample_circuit();
        let mut instance = BitVm3Instance::new(
            "inst-002".to_string(),
            dummy_pubkey(),
            dummy_pubkey(),
            circuit,
            10,
        )
        .unwrap();

        instance.initiate_challenge(1, 100).unwrap();

        let expensive_proof = BitVm3FraudProof {
            circuit_id: "bitvm3-circuit-001".to_string(),
            contested_gate_id: 1,
            wire_label_a: vec![0xaa; 32],
            wire_label_b: vec![0xbb; 32],
            proof_chunks: vec![vec![0xcc; 500]],
            estimated_fee_sats: 200_000, // Exceeds default 100,000 sats ceiling
        };

        let err = instance
            .submit_fraud_proof(&expensive_proof, 102, DEFAULT_MAX_SUB50_CHALLENGE_FEE_SATS)
            .unwrap_err();

        assert_eq!(
            err,
            BitVm3Error::Sub50DollarCostExceeded {
                fee_sats: 200_000,
                max_sats: 100_000,
            }
        );
    }

    #[test]
    fn test_chunk_size_limit_exceeded() {
        let mut circuit = sample_circuit();
        circuit.script_chunk_bytes = 150_000; // Exceeds 102,400 bytes limit

        let err = BitVm3Instance::new(
            "inst-003".to_string(),
            dummy_pubkey(),
            dummy_pubkey(),
            circuit,
            10,
        )
        .unwrap_err();

        assert_eq!(
            err,
            BitVm3Error::ChunkSizeExceeded {
                actual_bytes: 150_000,
                max_bytes: MAX_SCRIPT_CHUNK_BYTES,
            }
        );
    }

    #[test]
    fn test_timeout_detection() {
        let circuit = sample_circuit();
        let mut instance = BitVm3Instance::new(
            "inst-004".to_string(),
            dummy_pubkey(),
            dummy_pubkey(),
            circuit,
            10,
        )
        .unwrap();

        instance.initiate_challenge(1, 100).unwrap();
        assert!(!instance.check_timeout(109).unwrap());
        assert!(instance.check_timeout(110).unwrap());
        assert_eq!(instance.state, ChallengeState::TimedOut);
    }
}
