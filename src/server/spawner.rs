use std::collections::HashMap;
use std::net::{TcpStream, TcpListener};
use std::sync::{Mutex, Arc, mpsc, mpsc::Sender};
use std::thread::JoinHandle;
use std::vec;
use super::{diffuser, user_listener};

//The spawner thread is also the main thread 
//It manages incoming connections as well as joining the child threads for program closure

pub fn start(addr: &str){
    let (t_usr_handler, r_difusser) = mpsc::channel();
    
    let streams: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    
    let stop = Arc::new(Mutex::new(false));

    //Invoking clone on Arc produces a new Arc instance, which points to the same allocation on the heap as the source.
    let diffuser = diffuser::gen_diff(streams.clone(), r_difusser, stop.clone()); 
    let mut usr_handler_threads = vec![];

    usr_handler_threads = listen(streams, stop.clone(), addr, t_usr_handler, usr_handler_threads);

    join_threads(diffuser, usr_handler_threads);
}

fn listen(streams: Arc<Mutex<HashMap<usize, TcpStream>>>, stop: Arc<Mutex<bool>>, addr: &str, t_usr_handler: Sender<(usize, String)>, mut usr_handler_threads: Vec<JoinHandle<()>>) -> Vec<JoinHandle<()>>{
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    
    let mut id_counter: usize = 0;

    'outer: for stream_res in listener.incoming(){
        if *stop.lock().unwrap(){
            break 'outer;
        }
        
        match stream_res{
            Ok(new_stream) => {
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