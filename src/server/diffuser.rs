use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;
use std::sync::{Mutex, Arc, mpsc::Receiver};
use std::thread::{JoinHandle, self};
use std::time;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

//Child thread which handles incoming messages and diffuses them back to the users
//Also handles the processing of commands as they are treated as messages before reaching this thread

const COMM_CODE_DC: u8 = 101;
const COMM_CODE_FATAL_CONN_ERROR: u8 = 102;
const COMM_CODE_ACK: u8 = 103;
const COMM_CODE_STOP: u8 = 104;
const COMM_CODE_INVALID_COMMAND: u8 = 0;

pub fn gen_diff(streams: Arc<Mutex<HashMap<usize, TcpStream>>>, r_diffuser: Receiver<(usize, String)>, stop: Arc<Mutex<bool>>, priv_key: Arc<Mutex<RsaPrivateKey>>, pub_key: Arc<Mutex<RsaPublicKey>>, keys: Arc<Mutex<HashMap<usize, RsaPublicKey>>>) -> JoinHandle<()>{
    thread::spawn(move || {
        'outer: loop{
            let (tx_id, mut msg) = r_diffuser.recv().unwrap();

            //TODO: Decrypt messeage

            if msg.len()>2 && msg[0..=1] == *"//"{
                match handle_command(msg, &streams, tx_id){
                    COMM_CODE_STOP => {
                    terminate_connection(streams.clone(), tx_id, stop.clone());
                    break 'outer; 
                    },
                    _ => continue 'outer,
                }
            }

            msg = format!("{}: {}", tx_id, msg);

            msg = msg.replace(&['\n', '\0'], "").replace("//ACK", ""); //The write function can overlap itself and append 2 messeages together
            println!("{}", msg);

            match streams.lock(){
                Ok(mut streams_map) =>{  
                    for (id, stream) in (*streams_map).iter_mut(){
                        if *id == tx_id{
                            continue;
                        }
                        stream.write(msg.as_bytes()).unwrap();
                    }
                }
                Err(_) => (),
            }
        }
    })  
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

fn terminate_connection(streams: Arc<Mutex<HashMap<usize, TcpStream>>>, tx_id: usize, stop: Arc<Mutex<bool>>){
    match streams.lock(){
        Ok(mut streams_map) =>{
            (*streams_map).get(&tx_id).unwrap().set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            (*streams_map).remove(&tx_id);                  
            *stop.lock().unwrap() = true;
            
            for (_, stream) in (*streams_map).iter_mut(){
                stream.write("//STOP".as_bytes()).unwrap();
                stream.set_read_timeout(Some(time::Duration::from_millis(1000))).unwrap();
            }
        }
        Err(_) => (),
    }
}