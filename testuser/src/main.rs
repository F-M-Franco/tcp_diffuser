use p256::ecdsa::{SigningKey, Signature, signature::Signer, VerifyingKey, signature::Verifier};
use p256::{Scalar};
use rand_core::OsRng; // requires 'getrandom' feature
fn main(){

    let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
    println!("{:?}", signing_key.to_bytes());
    println!("{:?}", SigningKey::from_bytes(&signing_key.to_bytes()));
    let message = b"Golm";
    let signature: Signature = signing_key.sign(message);
    println!("{:?}", message);
    println!("{:?}", signature.to_bytes());
    let verifying_key = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
    println!("{:?}", verifying_key.to_encoded_point(false).as_bytes());
    println!("{:?}", VerifyingKey::from_sec1_bytes(verifying_key.to_encoded_point(false).as_bytes()));
    println!("{:?}", verifying_key.verify(message, &signature));
    
}
