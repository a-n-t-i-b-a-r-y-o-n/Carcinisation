use ed25519::Signature;
use ed25519::signature::Verifier;

pub struct PluginSignatureVerifier<V> {
    pub verifying_key: V
}

impl<V> PluginSignatureVerifier<V>
where
    V: Verifier<Signature>
{
    pub fn verify<C>(&self, content: C, signature: &Signature) -> Result<(), ed25519::Error>
    where
        C: AsRef<[u8]>
    {
        self.verifying_key.verify(content.as_ref(), signature)
    }
}