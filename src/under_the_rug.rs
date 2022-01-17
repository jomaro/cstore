
use axum::Router;
use axum::http::StatusCode;
use axum::AddExtensionLayer;
use tower_http::trace::TraceLayer;

use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};

#[derive(Clone)]
pub struct CryptoConfig {
    key: [u8; 32],
}

impl CryptoConfig {
    pub fn new(key: &str) -> Self {
        let mut config = CryptoConfig{
            key: [0u8; 32],
        };

        base64::decode_config_slice(key, base64::STANDARD, &mut config.key).unwrap();

        config
    }
}


pub fn extract_string_key(payload: &serde_json::Value, key: &str) -> Result<String, StatusCode> {
    return match payload.get(key) {
        Some(serde_json::Value::String(key)) => Ok(key.clone()),
        _ => Err(StatusCode::BAD_REQUEST)
    };
}

pub fn append_state(router: Router) -> Router {
    let db: sled::Db = sled::open("my_db.sled").unwrap();

    let crypto_config = CryptoConfig::new("0vLId2YLZnnALy3KYGtiYLI+yaSZ1rBNv+K2RzBdzNM=");
    router
        .layer(AddExtensionLayer::new(db))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(crypto_config))
}

fn get_nonce(_size: usize) -> Vec<u8> {
    vec![1,2,3,4,5,6,7,8,9,10,11,12]
}

pub fn encrypt(data: Vec<u8>, context: &CryptoConfig) -> Result<Vec<u8>, ()> {
    let key = Key::from_slice(&context.key);

    let cipher = ChaCha20Poly1305::new(key);

    let mut nonce = get_nonce(12);
    let prepared_nonce = Nonce::from_slice(&nonce);

    cipher.encrypt(&prepared_nonce, data.as_slice())
    .map(move |mut ciphertext| {nonce.append(&mut ciphertext); return nonce })
    .map_err(|_| ())
}

pub fn decrypt(data: Vec<u8>, context: &CryptoConfig) -> Result<Vec<u8>, ()> {
    let key = Key::from_slice(&context.key);

    let cipher = ChaCha20Poly1305::new(key);

    let (nonce, data) = data.as_slice().split_at(12);

    //let nonce = data[0..12];
    let prepared_nonce = Nonce::from_slice(&nonce);

    cipher.decrypt(prepared_nonce, data)
    .map_err(|_| ())
}
