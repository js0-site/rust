use rustls::pki_types::PrivateKeyDer;

fn main() {
    let key: PrivateKeyDer<'static> = PrivateKeyDer::Pkcs8(rustls::pki_types::PrivatePkcs8KeyDer::from(vec![]));
    let bytes: &[u8] = key.as_ref(); // Check if this compiles
}
