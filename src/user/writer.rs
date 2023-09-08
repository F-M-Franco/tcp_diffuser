use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt, Pkcs1v15Sign, Pss, pss};
use rand;
use rsa::pss::{BlindedSigningKey, VerifyingKey};
use rsa::signature::{Keypair,RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};

use super::reader;

// Writer thread is also the main thread
// It handles user input

pub fn connect(addr: &str){
    let mut stream = TcpStream::connect(addr).unwrap();
    
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    // TODO: pub_key to &[u8]
    stream.write(pub_key);
    let mut server_pub_key = [0; 2048]; 
    stream.read(&mut server_pub_key);
    // TODO: server_pub_key to RsaPublicKey
    
    let stop = Arc::new(Mutex::new(false));
    
    let reader = reader::gen_reader(stream.try_clone().unwrap(), stop.clone(), priv_key, server_pub_key); //try_clone => The returned TcpStream is a reference to the same stream that this object references. Both handles will read and write the same stream of data, and options set on one stream will be propagated to the other stream.
    
    handle_input(stream, stop.clone(), server_pub_key, priv_key);

    reader.join().unwrap();
}

fn handle_input(mut stream: TcpStream, stop: Arc<Mutex<bool>>, server_pub_key: RsaPublicKey, priv_key: RsaPrivateKey){    
    'outer: loop{
        if *stop.lock().unwrap() {            
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break 'outer;
        }

        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();
        
        let msg_encrypted = encrypt(msg.as_bytes(), &server_pub_key, &priv_key);

        match stream.write(&msg_encrypted){
            Ok(_) => (),
            Err(_) => {
                println!("Connection has been terminated. Press enter to exit");
                *stop.lock().unwrap() = true;
                break 'outer
            },
        };

        msg = msg.replace('\n', "");
        let line = msg.split(' ').collect::<Vec<&str>>();
        let line = line[0];

        if line == "//STOP" 
            || line == "//DC"{
            
            *stop.try_lock().unwrap() = true;
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break 'outer;
        }
    }
}

fn encrypt(msg: &[u8], server_pub_key: &RsaPublicKey, priv_key: &RsaPrivateKey) -> Vec<u8>{
    let mut rng = rand::thread_rng();

    let msg_encrypted = server_pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, msg).unwrap();
    
    let signing_key = BlindedSigningKey::<Sha256>::new(priv_key);
    let verifying_key = signing_key.verifying_key();

    let signature = signing_key.sign_with_rng(&mut rng, msg);
}