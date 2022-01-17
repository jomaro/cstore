
mod under_the_rug;

use std::str;

use axum::extract::Extension;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use axum::extract::Json;
use axum::http::StatusCode;

use base64;

use sled;

use under_the_rug::{
    extract_string_key, 
    append_state, 
    encrypt,
    decrypt,
    CryptoConfig,
};



async fn ping() -> &'static str {
    "health"
}

async fn store(
        Extension(db): Extension<sled::Db>, 
        Extension(crypto_context): Extension<CryptoConfig>,
        Json(raw_payload): Json<serde_json::Value>) -> Result<String, StatusCode> {
    let key: String = extract_string_key(&raw_payload, "key")?;
    let payload: Vec<u8> = 
        extract_string_key(&raw_payload, "payload")
            .map(base64::decode)
            .map_err(|_err| StatusCode::BAD_REQUEST)?.unwrap();

    println!("s {:?}", key);

    let encripted = encrypt(payload, &crypto_context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Ok(_) = db.insert(key.as_bytes(), encripted) {
        Ok("OK".to_string())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn get_key(
        Extension(db): Extension<sled::Db>,
        Extension(crypto_context): Extension<CryptoConfig>,
        Json(payload): Json<serde_json::Value>) -> Result<String, StatusCode> {

    println!("/store");
    let key: String = extract_string_key(&payload, "key")?;

    println!("g {:?}", key);

    

    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            match decrypt(value.to_vec(), &crypto_context){
                Ok(value) => Ok(base64::encode(value)),
                Err(()) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
                .route("/ping", get(ping))
                .route("/store", post(store))
                .route("/get", post(get_key));

    let app = append_state(app);

    println!("listening...");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}