mod handler;
mod user;
mod user_db;
mod chat;

use std::{io::Read, net::{TcpListener, TcpStream}, sync::Mutex, thread};

use atlas_shared::packet::{AtlasPacketReader, ATLAS_PACKET_SIZE};
use chat::ChatHandler;
use handler::PacketHandler;
use once_cell::sync::Lazy;
use user::UserHandler;
use user_db::UserDB;

static USER_DB: Lazy<Mutex<UserDB>> = Lazy::new(|| Mutex::new(UserDB::load("user_db.json".into())));

static USER_HANDLER: Lazy<Mutex<UserHandler>> = Lazy::new(|| Mutex::new(UserHandler {
    online: vec![],
}));

static CHAT_HANDLER: Lazy<Mutex<ChatHandler>> = Lazy::new(|| Mutex::new(ChatHandler {
    rooms: vec!["#chat".into()],
}));

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut data = [0; ATLAS_PACKET_SIZE];
        match stream.read(&mut data) {
            Ok(n) => {
                if n == 0 {
                    println!("Closing connection from {:?}", stream.peer_addr());
                    USER_HANDLER.lock().unwrap().remove_user(stream);
                    break;
                }

                let packet = data.get_packet().unwrap();
                PacketHandler::handle_packet(&mut stream, packet);
            },
            Err(err) => {
                println!("{}", err);
                USER_HANDLER.lock().unwrap().remove_user(stream);
                break;
            }
        }
    }
}

pub fn main() {
    let listener = TcpListener::bind("localhost:5678").expect("Failed to create listener");

    println!("Starting server...");

    USER_DB.lock().unwrap().save_db();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected {:?}", stream.peer_addr());
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}