# zkAttestations

**zkAttestations** provides a demonstration of how zero-knowledge proofs can be submitted and verified for Noir projects through attestations. This repository showcases a command-line interface (CLI) executable that facilitates various operations outlined below.

## Directory Structure

The repository is structured as a conventional Rust project. A dedicated `circuit` directory has been added, housing the Noir project files.

## Local Testing Environment

While the primary development environment for this project is a local network utilizing `anvil` or a `hardhat` node, adaptions can be made for testnet deployment with some adjustments.

The `eas` crate allows for interaction with the Ethereum Attestation Service contracts. To deploy them, use the command:

```bash
cargo run -- deploy
```

Executing the above will also create the `zkAttestation` attestation schema. This schema not only defines an interface for our attestations but also assigns a unique identifier to the project attestations. The schema is designed as:

```
bytes32 circuitId, bytes pubArgs, bytes proof
```

## Proof Submission Process

1. Initiate your work on the Noir project within the `circuit` directory.
2. Upon completion, ensure that both `circuit.proof` and `Verifier.toml` files are generated.
3. Execute the `attest` command to submit the attestation:

```bash
cargo run -- attest
```

The submitted attestation will encapsulate:

- `circuit_id`: `[u8;32]` - Represents the SHA2-256 hash derived from the `circuit.json` output file.
- `pub_param_bytes`: `[u8]` - Byte-encoded version of the `Verifier.toml` file, comprising the public verification parameters.
- `proof_bytes`: `[u8]` - Contains the generated proof.

## Proof Verification Process

For entities interested in verifying the proof:

1. Clone the repository version containing the circuit.
2. Execute the verification command:

```bash
cargo run -- verify
```

Post the generation of proof and verifier files, transition to the circuit directory and initiate the final verification step:

```bash
cd circuit
nargo verify
```

## Important Notes

- This repository serves purely as a proof of concept.
- It is not designed for deployment in production settings.
- The current implementation accommodates only a single proof per circuit. While it downloads every proof, this design choice aims to exemplify how multiple proofs for a given circuit can be obtained and verified.