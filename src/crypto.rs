use std::str::Utf8Error;

use aes::{
    cipher::{
        generic_array::GenericArray, inout::PadError, InvalidLength, KeyIvInit as _, StreamCipher,
    },
    Aes256,
};
use hmac::{Hmac, Mac};
use pbkdf2::{password_hash::SaltString, pbkdf2_hmac_array};
use rand::rngs::OsRng;
use sha2::{Sha256, Sha512};

type Aes256Ctr = ctr::Ctr128LE<Aes256>;
type HmacSha256 = Hmac<Sha256>;

pub fn encrypt(password: &str, data: &[u8], integrity: bool) -> Result<Vec<u8>, DeEncryptError> {
    // 🧂🍳 22 salty bytes
    let salt = SaltString::generate(&mut OsRng);

    // generate key
    let iv_key =
        pbkdf2_hmac_array::<Sha512, 48>(password.as_bytes(), salt.as_str().as_bytes(), 10_000);
    let iv = GenericArray::from_slice(&iv_key[..16]);
    let key = GenericArray::from_slice(&iv_key[16..]);

    let mut payload = data.to_vec();
    let mut cipher = Aes256Ctr::new(key, iv);

    cipher.apply_keystream(&mut payload);

    let mut encrypted: Vec<u8> = salt.to_string().into_bytes();

    if integrity {
        let mut hmac = HmacSha256::new_from_slice(key)?;
        hmac.update(&payload);
        let finalize = hmac.finalize().into_bytes();

        // [salt, hmac, payload]
        encrypted.extend_from_slice(&finalize);
        encrypted.extend_from_slice(&payload);
        return Ok(encrypted);
    }

    // [salt, payload]
    encrypted.extend_from_slice(&payload);
    Ok(encrypted)
}

pub fn decrypt(password: &str, data: &[u8], integrity: bool) -> Result<Vec<u8>, DeEncryptError> {
    // prevent panics
    if data.len() < 22 {
        return Err(DeEncryptError::DataTooSmall);
    }

    // Extract salt
    let (salt, data) = data.split_at(22);
    let salt = SaltString::from_b64(std::str::from_utf8(salt)?)?;

    // Generate key
    let iv_key =
        pbkdf2_hmac_array::<Sha512, 48>(password.as_bytes(), salt.as_str().as_bytes(), 10_000);
    let iv = GenericArray::from_slice(&iv_key[..16]);
    let key = GenericArray::from_slice(&iv_key[16..]);

    let mut payload;
    if integrity {
        // prevent panics
        if data.len() < 32 {
            return Err(DeEncryptError::DataTooSmall);
        }

        // Extract hmac and payload
        let (hmac, data) = data.split_at(32);
        payload = data.to_vec();

        // Verify hmac
        let mut mac = HmacSha256::new_from_slice(key)?;
        mac.update(&payload);
        mac.verify_slice(hmac)
            .map_err(|_| DeEncryptError::IntegrityError)?;
    } else {
        payload = data.to_vec();
    }

    // Decrypt payload
    let mut cipher = Aes256Ctr::new(key, iv);
    cipher.apply_keystream(&mut payload);

    Ok(payload)
}

#[derive(Debug, thiserror::Error)]
pub enum DeEncryptError {
    #[error("Failed to encrypt: {0}")]
    CipherError(PadError),
    #[error("Failed to decode bytes into utf8: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Password hash error: {0}")]
    SaltError(pbkdf2::password_hash::Error),
    #[error("Data integrity failure")]
    IntegrityError,
    #[error("Input data too small")]
    DataTooSmall,
    #[error("{0}")]
    HmacInvalidLength(#[from] InvalidLength),
}

impl From<PadError> for DeEncryptError {
    fn from(value: PadError) -> Self {
        Self::CipherError(value)
    }
}

impl From<pbkdf2::password_hash::Error> for DeEncryptError {
    fn from(value: pbkdf2::password_hash::Error) -> Self {
        Self::SaltError(value)
    }
}
