use ethers::utils::hex;
use std::fs;
use std::io::Error;

/// Default Noir proof path.
pub const PROOF_PATH: &'static str = "./circuit/proofs/circuit.proof";
/// Default Noir public parameters path.
pub const PUB_PARAMS: &'static str = "./circuit/Verifier.toml";

pub enum CircuitFile {
    Proof,
    PubParams,
}

pub fn load(file: CircuitFile) -> Result<Vec<u8>, Error> {
    match file {
        CircuitFile::Proof => {
            let hex_string = fs::read_to_string(PROOF_PATH)?;
            hex::decode(&hex_string).map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
        }
        CircuitFile::PubParams => fs::read(PUB_PARAMS),
    }
}

pub fn save(file: CircuitFile, data: &[u8]) -> Result<(), Error> {
    match file {
        CircuitFile::Proof => {
            let hex_string = hex::encode(data);
            fs::write(PROOF_PATH, hex_string)
        }
        CircuitFile::PubParams => fs::write(PUB_PARAMS, data),
    }
}
