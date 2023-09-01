use std::net::TcpStream;
use std::io::Write;
use std::sync::{Arc, Mutex};
use super::reader;

// Writer thread is also the main thread
// It handles user input

pub fn connect(addr: &str){
    let stream = TcpStream::connect(addr).unwrap();
    
    let stop = Arc::new(Mutex::new(false));
    
    let reader = reader::gen_reader(stream.try_clone().unwrap(), stop.clone()); //try_clone => The returned TcpStream is a reference to the same stream that this object references. Both handles will read and write the same stream of data, and options set on one stream will be propagated to the other stream.
    
    handle_input(stream, stop.clone());

    reader.join().unwrap();
}

fn handle_input(mut stream: TcpStream, stop: Arc<Mutex<bool>>){
    'outer: loop{
        if *stop.lock().unwrap() {            
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break 'outer;
        }

        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();
        
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