//! assemble and sign a raw SPL image as the FSBL image bootable by the SpacemiT K1 BROM

use std::{fs, path::Path};

use color_eyre::eyre::Result;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, LineEnding},
    pkcs1v15::Pkcs1v15Sign,
    traits::PublicKeyParts,
};
use sha2::{Digest, Sha256};

/// serves as both the ROTPK and SPL signature key,
/// can be re-generated arbitrarily if eFUSE is not burned
pub const KEY_FILE: &str = "rsakeypair0_prv.key";

// FSBL image layout constants
const MAGIC: &[u8; 4] = b"AIHD"; // image header magic
const RSA2048_BYTES: usize = 256; // size of RSA2048 modulus/signature
const HEADER_BYTES: usize = 32; // size of single AIHD header
const FSBL_HEADER_SIZE: usize = 0x1000; // total size of FSBL header (4 KB)
const SPL_ALIGN: usize = 32; // alignment of SPL data

// fixed constants for keydata and oem_key blocks
const KEY_NAMES: [(&[u8], u32); 4] = [
    // (name, id) for 4 key_tables
    (b"spl", 0),
    (b"uboot", 1),
    (b"kernel", 2),
    (b"rootfs", 3),
];
const KEYTABLE_NAME_BYTES: usize = 16; // size of each key_name field
const KEYDATA_RESERVED: usize = 320; // size of keydata reserved
const KEYDATA_TAIL_PAD: usize = 40; // size of keydata tail padding
const OEM_KEY_BYTES: usize = RSA2048_BYTES * 4 + 1024; // total size of oem_key block (2048)
const POST_CERT_PAD: usize = 992; // size of padding between cert0 and header1

pub fn load_or_generate_key(path: &Path) -> Result<RsaPrivateKey> {
    if path.exists() {
        return Ok(RsaPrivateKey::from_pkcs1_pem(&fs::read_to_string(path)?)?);
    }
    println!("==> generating RSA2048 signing key: {}", path.display());
    let key = RsaPrivateKey::new(&mut rand_core::OsRng, 2048)?;
    fs::write(path, key.to_pkcs1_pem(LineEnding::LF)?.as_bytes())?;
    Ok(key)
}

pub fn wrap(spl_raw: &[u8], prv: &RsaPrivateKey) -> Result<Vec<u8>> {
    let spl = align_pad(spl_raw, SPL_ALIGN); // align SPL data to 32 bytes
    let rotpk_mod = modulus_be(&RsaPublicKey::from(prv));

    let header0 = build_aihd_header(FSBL_HEADER_SIZE as u64);
    let keydata = build_keydata(&rotpk_mod);
    let oem_key = build_oem_key(&rotpk_mod); // only fill the spl slot, the rest are zero
    let signature0 = rsa_sign(prv, &[&header0, &keydata, &oem_key])?;

    let header1 = build_aihd_header(spl.len() as u64);
    let signature1 = rsa_sign(prv, &[&header1, &spl])?;

    let mut out = Vec::with_capacity(FSBL_HEADER_SIZE + spl.len() + RSA2048_BYTES);
    out.extend_from_slice(&rotpk_mod); // [0x000, 0x100): ROTPK
    out.extend_from_slice(&header0); // [0x100, 0x120): header0
    out.extend_from_slice(&keydata); // [0x120, 0x300): keydata
    out.extend_from_slice(&oem_key); // [0x300, 0xB00): oem_key
    out.extend_from_slice(&signature0); // [0xB00, 0xC00): signature0
    out.resize(out.len() + POST_CERT_PAD, 0); // [0xC00, 0xFE0): padding
    out.extend_from_slice(&header1); // [0xFE0, 0x1000): header1
    out.extend_from_slice(&spl); // original SPL data
    out.extend_from_slice(&signature1); // last 256 bytes signature
    debug_assert_eq!(out.len(), FSBL_HEADER_SIZE + spl.len() + RSA2048_BYTES);

    Ok(out)
}

// convert the modulus of RSA public key to a fixed length of 256 bytes in big endian (left pad to align with RSA2048)
fn modulus_be(pk: &RsaPublicKey) -> Vec<u8> {
    let raw = pk.n().to_bytes_be();
    let mut buf = vec![0u8; RSA2048_BYTES];
    buf[RSA2048_BYTES - raw.len()..].copy_from_slice(&raw);
    buf
}

// align the data length up to the specified number of bytes, padding with zeros if necessary
fn align_pad(data: &[u8], align: usize) -> Vec<u8> {
    let aligned = data.len().div_ceil(align) * align;
    let mut buf = Vec::with_capacity(aligned);
    buf.extend_from_slice(data);
    buf.resize(aligned, 0);
    buf
}

// build a 32-byte AIHD header: magic + version + secure + reserved + imgsize + load_addr + pad
fn build_aihd_header(img_size: u64) -> [u8; HEADER_BYTES] {
    let mut h = [0u8; HEADER_BYTES];
    h[0..4].copy_from_slice(MAGIC); // magic = "AIHD"
    h[4] = 1; // version = 1
    h[8..16].copy_from_slice(&img_size.to_le_bytes()); // imgsize, little endian u64
    h[24..32].fill(0xA5); // pad area fixed to 0xA5
    h
}

// build a 480-byte keydata block: key_default + table_num + 4×keytable + reserved + SHA256(ROTPK) + pad
fn build_keydata(rotpk_mod: &[u8]) -> Vec<u8> {
    let mut d = Vec::with_capacity(480);
    d.extend_from_slice(&0u32.to_le_bytes()); // key_default = 0
    d.extend_from_slice(&(KEY_NAMES.len() as u32).to_le_bytes()); // table_num = 4
    for (name, id) in KEY_NAMES {
        // 4 key_tables
        let mut slot = [0u8; KEYTABLE_NAME_BYTES];
        slot[..name.len()].copy_from_slice(name);
        d.extend_from_slice(&slot);
        d.extend_from_slice(&id.to_le_bytes());
    }
    d.resize(d.len() + KEYDATA_RESERVED, 0); // 320 bytes reserved area
    d.extend_from_slice(Sha256::digest(rotpk_mod).as_slice()); // SHA256 of ROTPK
    d.resize(d.len() + KEYDATA_TAIL_PAD, 0); // 40 bytes tail padding
    d
}

// build a 2048-byte oem_key block: slot0 = modulus of the public key used for signature, the rest are zero
fn build_oem_key(spl_pub_mod: &[u8]) -> Vec<u8> {
    let mut d = vec![0u8; OEM_KEY_BYTES];
    d[..RSA2048_BYTES].copy_from_slice(spl_pub_mod);
    d
}

// sign the concatenation of arbitrary data chunks using RSA PKCS#1 v1.5 SHA-256, return a fixed length of 256 bytes signature
fn rsa_sign(key: &RsaPrivateKey, chunks: &[&[u8]]) -> Result<Vec<u8>, rsa::Error> {
    let mut hasher = Sha256::new();
    for c in chunks {
        hasher.update(c);
    }
    let digest = hasher.finalize();
    key.sign(Pkcs1v15Sign::new::<Sha256>(), &digest)
}
