use std::fmt;
use aes_gcm::{AesGcm, Key, KeyInit, KeySizeUser, Nonce};
use aes_gcm::aead::consts::U16;
use aes_gcm::aead::Aead;
use aes_gcm::aes::Aes256;

use base64::Engine;
use base64::engine::general_purpose;
use rand::{distributions::Alphanumeric, Rng, thread_rng};

use crate::fromError;

pub(crate) struct Crypto {
    key: Key<AesGcm<Aes256,U16>>, //42
    nonce: Nonce<U16>, //16
}

impl Crypto {
    fn cipher(&self) -> AesGcm<Aes256, U16> {
        AesGcm::new_from_slice(self.key.as_ref()).unwrap()
    }

    #[allow(dead_code)]
    pub fn new_one(complex_key: String) -> Result<Self, CryptoError> {
        let key_size: usize = AesGcm::<Aes256, U16>::key_size();
        let nonce_size: usize = Nonce::<U16>::default().len();
        let joined_key_size = key_size + nonce_size;
        if complex_key.len() != joined_key_size {
            return Err(CryptoError::from(KeySizeError(format!(
                "Complex key has wrong length: {}. Expected: key({})+nonce({})",
                complex_key.len(),
                key_size,
                nonce_size,
            ))));
        }
        let key: &str = &complex_key[0..key_size];
        let nonce: &str = &complex_key[key_size..joined_key_size];
        Crypto::new(key.to_string(), nonce.to_string())
    }

    pub fn new(key: String, nonce: String) -> Result<Self, CryptoError> {
        let key_size: usize = AesGcm::<Aes256, U16>::key_size();
        let b_key = key.as_bytes();
        let b_nonce = nonce.as_bytes();
        if b_key.len() != key_size {
            return Err(CryptoError::from(KeySizeError(format!(
                "Key has wrong length: {}. Expected: {}",
                b_key.len(),
                key_size
            ))));
        }
        let nonce_size: usize = Nonce::<U16>::default().len();
        if b_nonce.len() != nonce_size {
            return Err(CryptoError::from(NonceSizeError(format!(
                "Nonce has wrong length: {}. Expected: {}",
                b_nonce.len(),
                nonce_size
            ))));
        }

        Ok(Crypto {
            key: Key::<AesGcm<Aes256, U16>>::from_slice(b_key).clone(),
            nonce: Nonce::<U16>::from_slice(&b_nonce).clone(),
        })
    }

    pub fn encrypt(&self, data: &String) -> String {
        let buffer = self.cipher().encrypt(&self.nonce,data.as_bytes())
            .expect("Encryption was unsuccessful");
        general_purpose::STANDARD.encode(buffer)
    }

    pub fn decrypt(&self, data: &String) -> String {
        let buffer = general_purpose::STANDARD.decode(data)
            .expect("String was not Base64 encoded");
        let result = self.cipher().decrypt(&self.nonce, buffer.as_slice())
            .expect("Unable to decrypt");
        String::from_utf8(result).expect("Unable to write bytes to string")
    }
}

#[derive(Debug)]
pub struct CryptoError(CryptoErrorKind);

impl CryptoError {
    pub(crate) fn new(kind: CryptoErrorKind) -> CryptoError {
        CryptoError(kind)
    }
}

#[derive(Debug)]
pub struct KeySizeError(String);

#[derive(Debug)]
pub struct NonceSizeError(String);

#[derive(Debug)]
pub struct Base64DecodeError(String);

#[derive(Debug)]
pub(crate) enum CryptoErrorKind {
    KeySizeError(KeySizeError),
    NonceSizeError(NonceSizeError),
    Base64DecodeError(Base64DecodeError),
}

fromError!(KeySizeError, CryptoError, CryptoErrorKind::KeySizeError);
fromError!(NonceSizeError, CryptoError, CryptoErrorKind::NonceSizeError);
fromError!(
    Base64DecodeError,
    CryptoError,
    CryptoErrorKind::Base64DecodeError
);

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            CryptoErrorKind::KeySizeError(error) => write!(f, "KeySizeError: {}", error.0),
            CryptoErrorKind::NonceSizeError(error) => write!(f, "NonceSizeError: {}", error.0),
            CryptoErrorKind::Base64DecodeError(error) => {
                write!(f, "Base64DecodeError: {}", error.0)
            }
        }
    }
}

impl std::error::Error for CryptoError {}

pub fn random_salt() -> String {
    let nonce_size: usize = Nonce::<U16>::default().len(); //16
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(nonce_size)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod test {
    use super::Crypto;

    #[test]
    pub fn test() {
        let key = "Some secret key!Some secret key!Some secret key!".to_owned();
        let message = "Some message to test crypto ".to_owned();
        let crypto = Crypto::new_one(key).unwrap();
        let enc = crypto.encrypt(&message);
        println!("Encrypted data: {}", enc);
        let dec = crypto.decrypt(&enc);
        let dec2 = crypto.decrypt(&enc);
        assert_eq!(message, dec);
        println!("Decrypted data: {}", dec);
        assert_eq!(message, dec2);
        println!("Decrypted data 2: {}", dec2);
    }
}
