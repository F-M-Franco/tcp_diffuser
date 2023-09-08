use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, Mutex};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rand;

use super::reader;

// Writer thread is also the main thread
// It handles user input

pub fn connect(addr: &str){
    let mut stream = TcpStream::connect(addr).unwrap();
    
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    // FIXME: stream.write(pub_key);
    let mut server_pub_key = [0; 2048]; 
    // FIXME: stream.read(server_pub_key);
    // TODO: user_pub_key to RsaPublicKey

    let stop = Arc::new(Mutex::new(false));
    
    // FIXME:let reader = reader::gen_reader(stream.try_clone().unwrap(), stop.clone(), server_pub_key); //try_clone => The returned TcpStream is a reference to the same stream that this object references. Both handles will read and write the same stream of data, and options set on one stream will be propagated to the other stream.
    
    handle_input(stream, stop.clone(), priv_key);

    reader.join().unwrap();
}

fn handle_input(mut stream: TcpStream, stop: Arc<Mutex<bool>>, priv_key: RsaPrivateKey){
    'outer: loop{
        if *stop.lock().unwrap() {            
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break 'outer;
        }

        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();
        
        // FIXME: let msg = priv_key.encrypt(msg);

        match stream.write(msg.as_bytes()){
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