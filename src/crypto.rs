use std::str::Utf8Error;

use aes::{
    cipher::{generic_array::GenericArray, InvalidLength, KeyIvInit as _, StreamCipher},
    Aes256,
};
use bincode::{
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use hmac::{Hmac, Mac};
use pbkdf2::{password_hash::SaltString, pbkdf2_hmac_array};
use rand::rngs::OsRng;
use sha2::{Sha256, Sha512};

type Aes256Ctr = ctr::Ctr128LE<Aes256>;
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Encode, Decode)]
struct Payload {
    integrity: bool,
    password_hmac: [u8; 32],
    salt: [u8; 22],
    hmac: Option<[u8; 32]>,
    data: Vec<u8>,
}

/// Check if stream was encrypted with integrity flag set
///
/// If stream is invalid, returns false
pub fn check_integrity(data: &[u8]) -> bool {
    let des = bincode::decode_from_slice::<Payload, _>(data, bincode::config::standard());

    if let Ok((payload, _)) = des {
        payload.integrity
    } else {
        false
    }
}

/// Encrypt a binary stream
///
/// Requirements:
/// - password.len > 0
pub fn encrypt(password: &str, data: &[u8], integrity: bool) -> Result<Vec<u8>, DeEncryptError> {
    if password.is_empty() {
        return Err(DeEncryptError::PasswordTooShort);
    }

    // 🧂🍳 22 salty bytes
    let salt = SaltString::generate(&mut OsRng);

    // generate key
    let iv_key =
        pbkdf2_hmac_array::<Sha512, 48>(password.as_bytes(), salt.as_str().as_bytes(), 10_000);
    let iv = GenericArray::from_slice(&iv_key[..16]);
    let key = GenericArray::from_slice(&iv_key[16..]);

    let mut data = data.to_vec();
    let mut cipher = Aes256Ctr::new(key, iv);
    cipher.apply_keystream(&mut data);

    let mut password_hmac = HmacSha256::new_from_slice(key)?;
    password_hmac.update(password.as_bytes());
    let password_hmac = password_hmac.finalize().into_bytes().into();

    let mut hmac = None;
    if integrity {
        let mut _hmac = HmacSha256::new_from_slice(key)?;
        _hmac.update(&data);
        hmac = Some(_hmac.finalize().into_bytes().into());
    }

    let payload = Payload {
        integrity,
        password_hmac,
        salt: salt.to_string().as_bytes().try_into().unwrap(),
        hmac,
        data,
    };

    let payload = bincode::encode_to_vec(payload, bincode::config::standard())?;

    Ok(payload)
}

/// Decrypt binary stream
///
/// Requirements:
/// - correct password
/// - password.len > 0
/// - integrity option must match what the data stream was encrypted with
/// - data is unaltered
///
/// Note: It is possible in rare cases decrypted data might pass successfully (particularly when not using
///       the integrity check). And in such a case the resulting returned data _may_ be corrupt.
pub fn decrypt(password: &str, data: &[u8], integrity: bool) -> Result<Vec<u8>, DeEncryptError> {
    if password.is_empty() {
        return Err(DeEncryptError::PasswordTooShort);
    }

    let (mut payload, _) =
        bincode::decode_from_slice::<Payload, _>(data, bincode::config::standard())?;

    // Incorrect param specified
    if integrity != payload.integrity {
        return Err(DeEncryptError::IncorrectIntegrity);
    }

    // Extract salt
    let salt = SaltString::from_b64(std::str::from_utf8(&payload.salt)?)?;

    // Generate key
    let iv_key =
        pbkdf2_hmac_array::<Sha512, 48>(password.as_bytes(), salt.as_str().as_bytes(), 10_000);
    let iv = GenericArray::from_slice(&iv_key[..16]);
    let key = GenericArray::from_slice(&iv_key[16..]);

    // verify password integrity
    let mut pwd_hmac = HmacSha256::new_from_slice(key)?;
    pwd_hmac.update(password.as_bytes());
    pwd_hmac
        .verify_slice(&payload.password_hmac)
        .map_err(|_| DeEncryptError::IncorrectPassword)?;

    if integrity {
        // Verify hmac
        let mut mac = HmacSha256::new_from_slice(key)?;
        mac.update(&payload.data);
        mac.verify_slice(&payload.hmac.unwrap())
            .map_err(|_| DeEncryptError::IntegrityError)?;
    }

    // Decrypt payload
    let mut cipher = Aes256Ctr::new(key, iv);
    cipher.apply_keystream(&mut payload.data);

    Ok(payload.data)
}

#[derive(Debug, thiserror::Error)]
pub enum DeEncryptError {
    #[error("Failed to decode bytes into utf8: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Password hash error: {0}")]
    SaltError(pbkdf2::password_hash::Error),
    #[error("Data integrity failure")]
    IntegrityError,
    #[error("{0}")]
    HmacInvalidLength(#[from] InvalidLength),
    #[error("Password must be non-zero length")]
    PasswordTooShort,
    #[error("Bincode encode failure: {0}")]
    EncodeError(#[from] EncodeError),
    #[error("Bincode decode failure: {0}")]
    DecodeError(#[from] DecodeError),
    #[error("Incorrect password entered")]
    IncorrectPassword,
    #[error("Integrity flag does not match the integrity of the underlying data")]
    IncorrectIntegrity,
}

impl From<pbkdf2::password_hash::Error> for DeEncryptError {
    fn from(value: pbkdf2::password_hash::Error) -> Self {
        Self::SaltError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pass() {
        let data = encrypt("123", &[1, 2, 3, 4], false).unwrap();
        let data = decrypt("123", &data, false).unwrap();

        assert_eq!(data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_integrity() {
        let data = encrypt("123", &[1, 2, 3, 4], true).unwrap();
        let data = decrypt("123", &data, true).unwrap();

        assert_eq!(data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_decrypt_integrity_broken() {
        let mut data = encrypt("123", &[1, 2, 3, 4], true).unwrap();
        data.pop();
        let data = decrypt("123", &data, true);

        assert!(data.is_err());
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let data = encrypt("123", &[1, 2, 3, 4], true).unwrap();
        let data = decrypt("1234", &data, true);

        assert!(data.is_err());
    }

    #[test]
    fn test_decrypt_wrong_integrity_flag() {
        let data = encrypt("123", &[1, 2, 3, 4], true).unwrap();
        let data = decrypt("1234", &data, false);

        assert!(data.is_err());

        let data = encrypt("123", &[1, 2, 3, 4], false).unwrap();
        let data = decrypt("1234", &data, true);

        assert!(data.is_err());
    }

    #[test]
    fn test_encrypt_no_pass() {
        assert!(encrypt("", &[1, 2, 3, 4], false).is_err());
    }

    #[test]
    fn test_decrypt_no_pass() {
        assert!(decrypt("", &[1, 2, 3, 4], false).is_err());
    }
}
