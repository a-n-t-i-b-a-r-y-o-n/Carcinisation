use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use ed25519::pkcs8::{DecodePrivateKey, DecodePublicKey};
use ed25519::Signature;
use ed25519::signature::{Signer, Verifier};
use ed25519_dalek::{SigningKey, VerifyingKey};

/// Public key used for verifying plugin signatures
pub(crate) const VERIFYING_KEY: &str =
r#"-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAChK1fAo0iK3VraSzLZGbZgB4oIKbkHjGIG+0Nzf1GMs=
-----END PUBLIC KEY-----"#;

/// Generic plugin signer
pub struct PluginSigner<S>
{
    pub signing_key: S
}

impl<S> PluginSigner<S>
where
    S: Signer<Signature>
{
    /// Sign the given content
    pub fn sign<C>(&self, content: C) -> Signature
    where
        C: AsRef<[u8]>
    {
        self.signing_key.sign(content.as_ref())
    }
}

/// Build a signer directly from a PEM private key file
impl<P> From<P> for PluginSigner<SigningKey>
where
    P: AsRef<OsStr> + AsRef<Path>,
{
    fn from(value: P) -> Self {
        let path = PathBuf::from(&value);
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        _ = file.read_to_string(&mut content).unwrap();
        let signing_key = SigningKey::from_pkcs8_pem(&content).unwrap();
        
        Self {
            signing_key,
        }
    }
}


/// Generic signature verifier
pub struct PluginSignatureVerifier<V> {
    pub verifying_key: V
}

impl<V> PluginSignatureVerifier<V>
where
    V: Verifier<Signature>
{
    /// Verify the signature of the given content
    pub fn verify<C>(&self, content: C, signature: &Signature) -> Result<(), ed25519::Error>
    where
        C: AsRef<[u8]>
    {
        self.verifying_key.verify(content.as_ref(), signature)
    }
}

/// Build a verifier directly from a PEM public key file
impl<P> From<P> for PluginSignatureVerifier<VerifyingKey>
where
    P: AsRef<OsStr> + AsRef<Path>,
{
    fn from(value: P) -> Self {
        let path = PathBuf::from(&value);
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        _ = file.read_to_string(&mut content).unwrap();
        let verifying_key = VerifyingKey::from_public_key_pem(&content).unwrap();

        Self {
            verifying_key,
        }
    }
}