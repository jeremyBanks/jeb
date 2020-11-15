mod utils;

use wasm_bindgen::prelude::*;
use aes_gcm::Aes256Gcm; // Or `Aes128Gcm`
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn aes_gcm_256_decrypt_and_verify_as_utf8(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> String {
    utils::set_panic_hook();

    assert!(key.len() == 32, "key length must be 32 bytes, but was {}", key.len());
    assert!(nonce.len() == 12, "nonce length must be 12 bytes, but was {}", nonce.len());

    let nonce = GenericArray::from_slice(nonce);
    let key = GenericArray::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .expect("AES-GCM decryption/verification failure");

    return String::from_utf8(plaintext).expect("utf-8 decoding failure");
}
