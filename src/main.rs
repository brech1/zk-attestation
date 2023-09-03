pub mod fs;
pub mod hash;

use crate::{
    fs::{load, CircuitFile},
    hash::hash_circuit,
};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use eas::{
    eas::*,
    eas_contracts::{
        eas::AttestedFilter,
        value_resolver::{Attestation, AttestationRequest, AttestationRequestData},
    },
    schema_registry::SchemaRegistryContract,
};
use ethers::{
    abi::RawLog,
    contract::EthEvent,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer},
    types::{Address, Bytes, U256},
};
use std::str::FromStr;

/// Ethereum provider URL.
const PROVIDER_URL: &str = "http://localhost:8545";
/// Chain ID.
const CHAIN_ID: u64 = 31337;
/// Default Mnemonic.
const DEFAULT_MNEMONIC: &'static str =
    "test test test test test test test test test test test junk";
/// Default EAS Contract Address.
const EAS_CONTRACT_ADDRESS: &str = "0xe7f1725e7734ce288f8367e1bb143e90bb3f0512";
/// Default Schema Registry Contract Address.
const _SCHEMA_REGISTRY_CONTRACT_ADDRESS: &'static str =
    "0x5fbdb2315678afecb367f032d93f642f64180aa3";
/// Default Schema ID.
const SCHEMA_ID: [u8; 32] = [
    232, 130, 60, 213, 42, 217, 24, 113, 178, 74, 194, 39, 104, 70, 212, 111, 139, 30, 76, 5, 150,
    91, 62, 157, 172, 70, 157, 108, 122, 94, 15, 169,
];
/// Default Schema.
const DEFAULT_SCHEMA: &'static str = "bytes32 circuitId, bytes pubArgs, bytes proof";
/// Byte array separator.
const SEPARATOR: [u8; 3] = [0x40, 0x40, 0x40];

/// CLI parser.
#[derive(Parser)]
pub struct Input {
    #[command(subcommand)]
    pub instruction: Instruction,
}

/// CLI instructions.
#[derive(Subcommand)]
pub enum Instruction {
    /// Attest generated proof.
    Attest,
    /// Deploy contracts.
    Deploy,
    /// Verify submitted proofs.
    Verify,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Read mnemonic from env
    let mnemonic = std::env::var("MNEMONIC").unwrap_or_else(|_| DEFAULT_MNEMONIC.to_string());

    // Setup provider
    let provider = Provider::<Http>::try_from(PROVIDER_URL)
        .expect("Failed to create provider from config node url");

    // Setup wallet
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic.as_str())
        .build()
        .expect("Failed to build wallet with provided mnemonic");

    // Setup signer
    let signer = SignerMiddleware::new(provider, wallet.with_chain_id(CHAIN_ID));

    match Input::parse().instruction {
        Instruction::Attest => {
            let eas = Eas::new(
                signer.clone(),
                Some(Address::from_str(EAS_CONTRACT_ADDRESS).unwrap()),
            );

            let circuit_id: [u8; 32] = hash_circuit().unwrap();

            println!("Circuit ID: {:?}", circuit_id);

            let mut pub_params_bytes: Vec<u8> = load(CircuitFile::PubParams).unwrap();
            let mut proof_bytes: Vec<u8> = load(CircuitFile::Proof).unwrap();

            let mut data_bytes: Vec<u8> = circuit_id.to_vec();
            data_bytes.append(&mut pub_params_bytes);
            data_bytes.append(&mut vec![0x40, 0x40, 0x40]); // Separator
            data_bytes.append(&mut proof_bytes);

            let att_data: Bytes = Bytes::from(data_bytes);

            let att_object = AttestationRequestData {
                recipient: Address::zero(),
                expiration_time: 0,
                revocable: false,
                ref_uid: [0u8; 32],
                data: att_data,
                value: U256::zero(),
            };

            let att = AttestationRequest {
                schema: SCHEMA_ID,
                data: att_object,
            };

            println!("Attestation: {:?}", att);

            eas.attest(att).await.unwrap();
        }
        Instruction::Deploy => {
            // Start EAS Contract and Schema Registry
            let mut eas_contract = Eas::new(signer.clone(), None);
            let mut schema_registry = SchemaRegistryContract::new(signer.clone(), None);

            // Deploy contracts
            let registry_address = schema_registry.deploy().await.unwrap();
            let eas_contract = eas_contract.deploy(registry_address).await.unwrap();

            println!("EAS Contract: {:?}", eas_contract);
            println!("Schema Registry: {:?}", registry_address);

            // Register schema
            let schema_id = schema_registry
                .register_schema(DEFAULT_SCHEMA.to_string(), Address::zero(), true)
                .await
                .unwrap();

            println!("Schema ID: {:?}", schema_id);
        }
        Instruction::Verify => {
            let eas_contract = Eas::new(
                signer.clone(),
                Some(Address::from_str(EAS_CONTRACT_ADDRESS).unwrap()),
            );

            let circuit_id: [u8; 32] = hash_circuit().unwrap();

            println!("Circuit ID: {:?}", circuit_id);

            let attested_filter = eas_contract.eas().attested_filter().filter.from_block(0);

            let logs = eas_contract
                .signer()
                .get_logs(&attested_filter)
                .await
                .unwrap();

            let decoded_logs: Vec<AttestedFilter> = logs
                .iter()
                .map(|log| {
                    let raw_log = RawLog::from((log.topics.clone(), log.data.to_vec()));
                    AttestedFilter::decode_log(&raw_log).unwrap()
                })
                .collect();

            // Store all attestation IDs
            let mut attestation_ids: Vec<[u8; 32]> = Vec::new();
            for log in decoded_logs {
                attestation_ids.push(log.uid);
            }

            println!("Attestation IDs: {:?}", attestation_ids);

            // Get all attestations
            let mut attestation_data: Vec<Attestation> = Vec::new();
            for attestation_id in attestation_ids {
                let attestation = eas_contract
                    .eas()
                    .get_attestation(attestation_id)
                    .call()
                    .await
                    .unwrap();

                attestation_data.push(attestation);
            }

            println!("Attestations: {:?}", attestation_data);

            for attestation in attestation_data {
                // Decode attestation payload bytes.
                let mut circuit_id_bytes: [u8; 32] = [0u8; 32];
                circuit_id_bytes.copy_from_slice(&attestation.data[0..32]);

                // Compare circuit ID bytes with the one generated from the circuit.
                if circuit_id_bytes != circuit_id {
                    println!("Circuit ID mismatch");
                    continue;
                }

                // Find separator.
                let id = attestation
                    .data
                    .windows(SEPARATOR.len())
                    .position(|window| window == &SEPARATOR);

                if id.is_none() {
                    println!("Separator not found");
                    continue;
                }

                let separator_index = id.unwrap();

                // Get pub params bytes.
                let pub_params_bytes = attestation.data[32..separator_index].to_vec();
                // Get proof bytes.
                let proof_bytes = attestation.data[separator_index + SEPARATOR.len()..].to_vec();

                // Generate files
                fs::save(CircuitFile::PubParams, pub_params_bytes.as_slice()).unwrap();
                fs::save(CircuitFile::Proof, proof_bytes.as_slice()).unwrap();
            }
        }
    }
}
