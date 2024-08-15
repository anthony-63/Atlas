use std::net::{SocketAddr, TcpStream};

use crate::user_db::UserEntry;

#[repr(u16)]
pub enum UserPermissions {
    ChatMod = 1 << 0,
    UserMod = 1 << 1,
}

pub struct User {
    pub user: UserEntry,
    pub token: String,
    pub status: String,
    pub stream: TcpStream,
    pub joined_room: String,
}

pub struct UserHandler {
    pub online: Vec<User>,
}

impl UserHandler {
    pub fn add_user(&mut self, stream: &TcpStream, entry: &UserEntry) -> String {
        println!("Logged in user: {} with addr {:?}", entry.name, stream.peer_addr());

        let token = format!("{:?}", gxhash::gxhash64(&entry.id.to_be_bytes(), 1234));

        self.online.push(User {
            stream: (*stream).try_clone().unwrap(),
            status: "None".into(),
            token: token.clone(),
            user: (*entry).clone(),
            joined_room: "none".into(),
        });

        return token;
    }

    pub fn is_online(&mut self, token: String) -> bool {
        match self.online.iter().find(|user| user.token == token) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_user_index(&mut self, token: String) -> usize {
        return self.online.iter().position(|user| user.token == token).unwrap();
    }

    pub fn set_room(&mut self, token: String, room: String) -> bool {
        match self.online.iter_mut().find(|user| user.token == token) {
            Some(user) => {
                user.joined_room = room;
                return true;
            }
            None => return false,
        }
    }

    pub fn remove_user(&mut self, stream: TcpStream) {
        let entry = match self.online.iter().find(|user| user.stream.peer_addr().unwrap() == stream.peer_addr().unwrap()) {
            Some(e) => e,
            None => return,
        };

        println!("Removed user: {} with addr {:?}", entry.user.name, stream.peer_addr());
        self.online.remove(self.online.iter().position(|user| user.stream.peer_addr().unwrap() == stream.peer_addr().unwrap()).unwrap());
    }
}
