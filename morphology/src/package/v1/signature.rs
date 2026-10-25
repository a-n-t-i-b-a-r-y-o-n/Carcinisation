use ed25519::Signature;
use ed25519::signature::{Signer, Verifier};

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