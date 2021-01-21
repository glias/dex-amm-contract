use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use blake2b_ref::{Blake2b, Blake2bBuilder};
use bytes::Bytes;

const INFO_LOCK_CODE_HASH_FILENAME: &str = "info_lock_code_hash.rs";
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

fn main() {
    let path = if cfg!(debug_assertions) {
        Path::new("../../build/debug/info-lock-script")
    } else {
        Path::new("../../build/release/info-lock-script")
    };

    let info_lock_bin = Bytes::from(fs::read(path).unwrap());

    let mut hash = [0u8; 32];
    let mut hasher = new_blake2b();
    hasher.update(&info_lock_bin);
    hasher.finalize(&mut hash);

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join(INFO_LOCK_CODE_HASH_FILENAME);
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(&mut file, "{:?}", hex::encode(hash)).unwrap();
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}
