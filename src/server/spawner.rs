use std::collections::HashMap;
use std::net::{TcpStream, TcpListener};
use std::sync::{Mutex, Arc, mpsc, mpsc::Sender};
use std::thread::JoinHandle;
use std::io::{Write, Read};
use std::vec;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rand;
use super::{diffuser, user_listener};
use serde::{Serialize, Deserialize};
//The spawner thread is also the main thread 
//It manages incoming connections as well as joining the child threads for program closure

pub fn start(addr: &str){
    let (t_usr_handler, r_difusser) = mpsc::channel();
    
    let streams: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    
    let mut rng = rand::thread_rng();
    let priv_key: Arc<Mutex<RsaPrivateKey>> = Arc::new(Mutex::new(RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate a key")));
    let pub_key: Arc<Mutex<RsaPublicKey>> = Arc::new(Mutex::new(RsaPublicKey::from(&*priv_key.lock().unwrap())));
   
    let keys: Arc<Mutex<HashMap<usize, RsaPublicKey>>> = Arc::new(Mutex::new(HashMap::new()));

    let stop = Arc::new(Mutex::new(false));

    //Invoking clone on Arc produces a new Arc instance, which points to the same allocation on the heap as the source.
    let diffuser = diffuser::gen_diff(streams.clone(), r_difusser, stop.clone(), priv_key.clone(), keys.clone()); 
    let mut usr_handler_threads = vec![];

    usr_handler_threads = listen(streams, stop, addr, t_usr_handler, usr_handler_threads, pub_key, keys);

    join_threads(diffuser, usr_handler_threads);
}

fn listen(streams: Arc<Mutex<HashMap<usize, TcpStream>>>, stop: Arc<Mutex<bool>>, addr: &str, t_usr_handler: Sender<(usize, &[u8])>, mut usr_handler_threads: Vec<JoinHandle<()>>, pub_key: Arc<Mutex<RsaPublicKey>>, keys: Arc<Mutex<HashMap<usize, RsaPublicKey>>>) -> Vec<JoinHandle<()>>{
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    
    let mut id_counter: usize = 0;

    'outer: for stream_res in listener.incoming(){
        if *stop.lock().unwrap(){
            break 'outer;
        }

        match stream_res{
            Ok(new_stream) => {
                let mut user_pub_key: [u8; 2048] = [0; 2048]; 
                new_stream.read(&mut user_pub_key);
                // TODO: user_pub_key to RsaPublicKey
                // TODO: pub_key to &[u8]
                (*pub_key.lock().unwrap());
                    
                new_stream.write(&*pub_key.lock().unwrap());

                (*keys.lock().unwrap()).insert(id_counter, user_pub_key);

                streams.lock().unwrap().insert(id_counter, new_stream.try_clone().unwrap());                
                usr_handler_threads.push(user_listener::gen_usr_handler(new_stream, t_usr_handler.clone(), stop.clone(), id_counter));
                
                id_counter+=1;
            },
            Err(_) => continue,
        }
        
    }
    return usr_handler_threads
}

fn join_threads(diffuser: JoinHandle<()>, usr_handler_threads: Vec<JoinHandle<()>>) {
    diffuser.join().unwrap();
    for thread in usr_handler_threads{
        thread.join().unwrap();
    }
}