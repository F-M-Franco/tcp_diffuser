use std::time;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, self};
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};

pub fn gen_reader(mut stream: TcpStream, stop: Arc<Mutex<bool>>, priv_key: RsaPrivateKey, server_pub_key: RsaPublicKey) -> JoinHandle<()>{
    thread::spawn(move || {
        let mut rng = rand::thread_rng();

        'outer: loop{
            //Sleep here prevents thread from getting ahead of main thread when its closing the connection
            thread::sleep(time::Duration::from_millis(375));

            if *stop.lock().unwrap() {
                break 'outer;
            }

            let mut msg: [u8; 2048] = [0; 2048];
            match stream.read(&mut msg){
                Ok(_) => (),
                Err(_) => {
                    println!("Connection has been terminated. Press enter to exit");
                    *stop.lock().unwrap() = true;
                    break 'outer
                },
            }

            if !check_validity(&msg, server_pub_key){
                continue;
            }

            let msg = priv_key.decrypt(Pkcs1v15Encrypt, &msg).unwrap();
            
            let msg = String::from_utf8(msg).unwrap();
            let msg = msg.replace(&['\n', '\0'], "").replace("//ACK", "");
            
            if msg == "//STOP" {
                println!("Connection has been closed manually. Press enter to exit.");
                *stop.try_lock().unwrap() = true;
                break 'outer;
            }

            println!("{}", msg);

            let ack = server_pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, "//ACK".as_bytes()).unwrap();
            let ack = priv_key.sign(Pkcs1v15Encrypt, ack).unwrap();

            match stream.write(&ack.into_boxed_slice()){
                Ok(_) => (),
                Err(_) => {
                    println!("Connection has been terminated. Press enter to exit");
                    *stop.lock().unwrap() = true;
                    break 'outer
                },
            }
             
        }
    })
}

fn check_validity(msg: &[u8], server_pub_key: RsaPublicKey) -> bool{
    match server_pub_key.verify(scheme, hashed, sig){
        Ok(_) => return true,
        Err(_) => return false,
    }
}