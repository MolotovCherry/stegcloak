use stegcloak::encrypt;
use stegcloak::plaintext;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn plaintext_hide(secret: &str, message: &str) -> Result<String, JsError> {
    Ok(plaintext::hide(secret, message)?)
}

#[wasm_bindgen]
pub fn plaintext_reveal(message: &str) -> Result<String, JsError> {
    Ok(plaintext::reveal(message)?)
}

#[wasm_bindgen]
pub fn encrypt_hide(
    secret: &str,
    password: &str,
    integrity: bool,
    message: &str,
) -> Result<String, JsError> {
    Ok(encrypt::hide(secret, password, integrity, message)?)
}

#[wasm_bindgen]
pub fn encrypt_reveal(password: &str, message: &str) -> Result<String, JsError> {
    Ok(encrypt::reveal(password, message)?)
}
