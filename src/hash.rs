use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, Read};
use walkdir::WalkDir;

const CIRCUIT_PATH: &'static str = "../circuit/target";

pub fn hash_circuit() -> Result<[u8; 32], io::Error> {
    let mut hasher = Sha256::new();

    for entry in WalkDir::new(CIRCUIT_PATH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let mut file = BufReader::new(File::open(entry.path())?);
        let mut buffer = [0u8; 1024];
        while let Ok(count) = file.read(&mut buffer) {
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}
