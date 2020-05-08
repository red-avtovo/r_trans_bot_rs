use aes::Aes256;
use base64::{encode, decode};
use ofb::Ofb;
use ofb::stream_cipher::{NewStreamCipher, SyncStreamCipher};
use aes::block_cipher_trait::{
    BlockCipher,
    generic_array::typenum::Unsigned
};
use std::fmt;
use crate::fromError;

pub(crate) struct Crypto {
    key: Vec<u8>,
    iv: Vec<u8>,
}

pub type AesOfb = Ofb<Aes256>;

pub trait EncSize {
    fn key_size() -> usize;
    fn nonce_size() -> usize;
}

impl<C: BlockCipher> EncSize for Ofb<C> {
    fn key_size() -> usize {
        <C::KeySize as Unsigned>::to_usize()
    }

    fn nonce_size() -> usize {
        <C::BlockSize as Unsigned>::to_usize()
    }
}


impl Crypto {

    fn cipher(&self) -> AesOfb {
        AesOfb::new_var(self.key.as_ref(), self.iv.as_ref()).unwrap()
    }

    pub fn new_one(complex_key: String) -> Result<Self, CryptoError> {
        if complex_key.len() != AesOfb::key_size()+AesOfb::nonce_size() {
            return Err(CryptoError::from(KeySizeError(
                format!("Complex key has wrong length: {}. Expected: key({})+nonce({})", complex_key.len(), AesOfb::key_size(), AesOfb::nonce_size())
            )));
        }
        let key: &str = &complex_key[0..32]; // u32
        let iv : &str = &complex_key[32..48]; // u16
        let value = 5;
        let a = vec![0; value].into_boxed_slice();
        println!("Key size: {}", &key.len());
        println!("IV size: {}", &iv.len());
        Crypto::new(
            key.to_string(),
            iv.to_string()
        )
    }

    pub fn new(key: String, iv: String) -> Result<Self, CryptoError> {
        let b_key = key.as_bytes();
        let b_iv = iv.as_bytes();
        if b_key.len() != AesOfb::key_size() { return Err(CryptoError::from(KeySizeError(
            format!("Key has wrong length: {}. Expected: {}", b_key.len(), AesOfb::key_size())
        ))) }
        if b_iv.len() != AesOfb::nonce_size() { return Err(CryptoError::from(NonceSizeError(
            format!("Nonce(iv) has wrong length: {}. Expected: {}", b_iv.len(), AesOfb::nonce_size())
        ))) }
        Ok(Crypto {
            key: b_key.to_vec(),
            iv: b_iv.to_vec()
        })
    }

    pub fn encrypt(&self, data: &String) -> String {
        let mut buffer = data.as_bytes().to_vec();
        self.cipher().apply_keystream(&mut buffer);
        encode(buffer)
    }

    pub fn decrypt(&self, data: &String) -> String {
        let mut buffer = decode(data).expect("String was not Base64 encoded");
        self.cipher().apply_keystream(&mut buffer);
        String::from_utf8(buffer).expect("Unable to write bytes to string")
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
pub(crate) enum CryptoErrorKind{
    KeySizeError(KeySizeError),
    NonceSizeError(NonceSizeError),
    Base64DecodeError(Base64DecodeError)
}

fromError!(KeySizeError, CryptoError, CryptoErrorKind::KeySizeError);
fromError!(NonceSizeError, CryptoError, CryptoErrorKind::NonceSizeError);
fromError!(Base64DecodeError, CryptoError, CryptoErrorKind::Base64DecodeError);

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            CryptoErrorKind::KeySizeError(error) => write!(f, "KeySizeError: {}", error.0),
            CryptoErrorKind::NonceSizeError(error) => write!(f, "NonceSizeError: {}", error.0),
            CryptoErrorKind::Base64DecodeError(error) => write!(f, "Base64DecodeError: {}", error.0),
        }
    }
}

impl std::error::Error for CryptoError{}

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