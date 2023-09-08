use std::time;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, self};
use rsa::RsaPublicKey;

pub fn gen_reader(mut stream: TcpStream, stop: Arc<Mutex<bool>>, server_pub_key: RsaPublicKey) -> JoinHandle<()>{
    thread::spawn(move || {
        'outer: loop{
            //Sleep here prevents thread from getting ahead of main thread when its closing the connection
            thread::sleep(time::Duration::from_millis(375));

            if *stop.lock().unwrap() {
                break 'outer;
            }

            let mut buf = [0; 255];
            match stream.read(&mut buf){
                Ok(_) => (),
                Err(_) => {
                    println!("Connection has been terminated. Press enter to exit");
                    *stop.lock().unwrap() = true;
                    break 'outer
                },
            }

            let msg = String::from_utf8(buf.to_vec()).unwrap();
            // FIXME: let msg = server_pub_key.decrypt(msg);
            let msg = msg.replace(&['\n', '\0'], "").replace("//ACK", "");
            
            if msg == "//STOP" {
                println!("Connection has been closed manually. Press enter to exit.");
                *stop.try_lock().unwrap() = true;
                break 'outer;
            }

            println!("{}", msg);

            match stream.write("//ACK".as_bytes()){
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