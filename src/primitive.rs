pub const PUBLIC_KEY_SIZE: usize = 32;
pub const HASH_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;

pub type PublicKey = [u8; PUBLIC_KEY_SIZE];
pub type Hash = [u8; HASH_SIZE]; // hash also is defined in blake2s
pub type Signature = [u8; SIGNATURE_SIZE];
