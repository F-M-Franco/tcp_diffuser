use std::net::TcpStream;
use std::io::Read;
use std::sync::{Mutex, Arc, mpsc::Sender};
use std::thread::{JoinHandle, self};
use std::time;
    
//This child thread listens for incoming messeages from a user after spawner begins the conversation

pub fn gen_usr_handler(mut stream: TcpStream, t_usr_handler: Sender<(usize, &[u8])>, stop: Arc<Mutex<bool>>, id: usize) -> JoinHandle<()>{
    thread::spawn(move || {
        'outer: loop{
            //Sleep here prevents thread from getting ahead of diffuser thread when it handles the STOP command
            thread::sleep(time::Duration::from_millis(375));

            if *stop.lock().unwrap()
                || stream.read_timeout().unwrap() != None{
                stream.shutdown(std::net::Shutdown::Both).unwrap();
                break 'outer;   
            };

            let mut buf = [0; 2048];

            match stream.read(&mut buf){
                Ok(_) => {
                    match t_usr_handler.send((id, &buf)){
                        Ok(_) => (),
                        Err(_) => (), //Error occures only when handling a STOP command called by a different connection
                    }
                },
                // FIXME: If the user/peer dies the error arm should trigger due to a connection error when trying to read from a non existant client
                //To handle the error we send a message to the diffuser using the command prefix of '//'
                Err(_) => { 
                    t_usr_handler.send((id, "//FATAL.CONN.ERROR".as_bytes())).unwrap();
                    break 'outer;
                },
            }
        }
    })
}