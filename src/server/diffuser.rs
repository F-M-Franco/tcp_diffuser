use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;
use std::sync::{Mutex, Arc, mpsc::Receiver};
use std::thread::{JoinHandle, self};
use std::time;
use rand::rngs::ThreadRng;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt, Pss};

//Child thread which handles incoming messages and diffuses them back to the users
//Also handles the processing of commands as they are treated as messages before reaching this thread

const COMM_CODE_DC: u8 = 101;
const COMM_CODE_FATAL_CONN_ERROR: u8 = 102;
const COMM_CODE_ACK: u8 = 103;
const COMM_CODE_STOP: u8 = 104;
const COMM_CODE_INVALID_COMMAND: u8 = 0;
const RNG: ThreadRng = rand::thread_rng();

pub fn gen_diff(streams: Arc<Mutex<HashMap<usize, TcpStream>>>, r_diffuser: Receiver<(usize, &[u8])>, stop: Arc<Mutex<bool>>, priv_key: Arc<Mutex<RsaPrivateKey>>, keys: Arc<Mutex<HashMap<usize, RsaPublicKey>>>) -> JoinHandle<()>{
    thread::spawn(move || { 
        'outer: loop{
            let (tx_id, mut msg) = r_diffuser.recv().unwrap();

            if !check_validity(tx_id, &keys, msg){
                continue;
            }

            let msg = (*priv_key.lock().unwrap()).decrypt(Pkcs1v15Encrypt, &msg).unwrap();

            let mut msg = String::from_utf8(msg).unwrap();

            if msg.len()>2 && msg[0..=1] == *"//"{
                match handle_command(msg, &streams, tx_id){
                    COMM_CODE_STOP => {
                        terminate_connection(&streams, tx_id, &stop, &keys, &priv_key);
                        break 'outer; 
                    },
                    _ => continue 'outer,
                }
            }

            diffuse(msg, &streams, tx_id, &keys, &priv_key);
        }
    })  
}

fn check_validity(tx_id: usize, keys: &Arc<Mutex<HashMap<usize, RsaPublicKey>>>, msg: &[u8]) -> bool{
    

    match (*keys.lock().unwrap()).get(&tx_id).unwrap().verify(scheme, hashed, sig){
        Ok(_) => return true,
        Err(_) => return false,
    }
}

fn diffuse(msg: String, streams: &Arc<Mutex<HashMap<usize, TcpStream>>>, tx_id: usize, keys: &Arc<Mutex<HashMap<usize, RsaPublicKey>>>, priv_key: &Arc<Mutex<RsaPrivateKey>>){
    msg = format!("{}: {}", tx_id, msg);

    msg = msg.replace(&['\n', '\0'], "").replace("//ACK", ""); //The write function can overlap itself and append 2 messeages together
    println!("{}", msg);
    match streams.lock(){
        Ok(mut streams_map) =>{  
            for (id, stream) in (*streams_map).iter_mut(){
                if *id == tx_id{
                    continue;
                }
                
                let msg_encrypted = (*keys.lock().unwrap()).get(&id).unwrap().encrypt(&mut RNG.clone(), Pkcs1v15Encrypt, &msg.as_bytes()).unwrap();
                let msg_encrypted = (*priv_key.lock().unwrap()).sign(Pss::new_with_salt::<_>(8), &msg_encrypted).unwrap();

                stream.write(msg.as_bytes()).unwrap();
            }
        }
        Err(_) => (),
    }
}

fn handle_command(mut msg: String, streams: &Arc<Mutex<HashMap<usize, TcpStream>>>, tx_id: usize) -> u8{
    msg = msg.replace(&['/', '\n', '\0'], "");
    let line = msg.split(' ').collect::<Vec<&str>>();
    let command = line[0];

    match command{
        "FATAL.CONN.ERROR" => {
            println!("{}: //{}", tx_id, command);
            streams.lock().unwrap().remove(&tx_id).unwrap().set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            return COMM_CODE_FATAL_CONN_ERROR;
        },
        "DC" => {
            println!("{}: //{}", tx_id, command);   
            streams.lock().unwrap().remove(&tx_id).unwrap().set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            return COMM_CODE_DC;
        }
        "ACK" => return COMM_CODE_ACK,
        
        "STOP" => {
            println!("{}: //{}", tx_id, command);   
            return COMM_CODE_STOP;
        },

        _ => return COMM_CODE_INVALID_COMMAND,
    }
}

fn terminate_connection(streams: &Arc<Mutex<HashMap<usize, TcpStream>>>, tx_id: usize, stop: &Arc<Mutex<bool>>, keys: &Arc<Mutex<HashMap<usize, RsaPublicKey>>>, priv_key: &Arc<Mutex<RsaPrivateKey>>){
    match streams.lock(){
        Ok(mut streams_map) =>{
            (*streams_map).get(&tx_id).unwrap().set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            (*streams_map).remove(&tx_id);                  
            *stop.lock().unwrap() = true;
            
            for (id, stream) in (*streams_map).iter_mut(){
                let msg_encrypted = (*keys.lock().unwrap()).get(&id). unwrap().encrypt(&mut RNG.clone(), Pkcs1v15Encrypt, &"//STOP".as_bytes()).unwrap();
                let msg_encrypted = (*priv_key.lock().unwrap()).sign(Pss::new_with_salt::<_>(8), &msg_encrypted).unwrap();
                
                stream.write(&*msg_encrypted.into_boxed_slice()).unwrap();
                stream.set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            }
        }
        Err(_) => (),
    }
}