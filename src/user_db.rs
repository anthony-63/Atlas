use std::{fs::File, io::{Read, Seek, Write}, net::TcpStream, path::Path};

use atlas_shared::packet::{AtlasPacket, AtlasResponseCodes};
use groestl::{Digest, Groestl256};
use serde::{Deserialize, Serialize};

use crate::USER_HANDLER;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserEntry {
    pub name: String,
    pub password_hash: String,
    pub id: usize
}

pub struct UserDB {
    pub file: std::fs::File,
    pub db: Vec<UserEntry>,
    pub current_id: usize,
}

impl UserDB {
    pub fn load(path: String) -> Self {
        let mut db: Vec<UserEntry> = vec![];
        let mut current_id = 0;

        let file = if Path::new(&path).exists() {
            let mut file = File::options().read(true).write(true).open(path).unwrap();

            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents).unwrap();
            db = serde_json::from_str(&file_contents).unwrap();
            if db.len() > 0 {
                current_id = db.iter().map(|user| user.id).max().unwrap();
            }

            file
        } else {
            File::create(&path).unwrap();
            File::options().read(true).truncate(true).write(true).open(path).unwrap()
        };

        Self {
            file,
            db,
            current_id
        }
    }

    pub fn save_db(&mut self) {
        self.file.set_len(0).unwrap();
        self.file.rewind().unwrap();
        self.file.write(serde_json::to_string_pretty(&self.db).unwrap().as_bytes()).unwrap();
    }

    pub fn register(&mut self, stream: &mut TcpStream, username: String, password: String) {
        let username_exists = match self.db.iter().find(|user| user.name == username) {
            Some(_) => true,
            None => false,
        };

        if username_exists {
            stream.write_all(&AtlasPacket::RegisterResponse(AtlasResponseCodes::UsernameExists as u16).serialize().unwrap()).unwrap();
            return;
        }

        let mut hasher = Groestl256::default();
        hasher.update(password);
        let hash = format!("{:x}", hasher.finalize());

        self.current_id +=  1;

        self.db.push(UserEntry {
            id: self.current_id,
            name: username,
            password_hash: hash,
        });

        self.save_db();

        stream.write_all(&AtlasPacket::RegisterResponse(AtlasResponseCodes::Success as u16).serialize().unwrap()).unwrap();
    }

    pub fn login(&mut self, stream: &mut TcpStream, username: String, password: String) {
        let username_exists = match self.db.iter().find(|user| user.name == username) {
            Some(_) => true,
            None => false,
        };

        if !username_exists {
            stream.write_all(&AtlasPacket::LoginResponse(AtlasResponseCodes::UserDoesntExist as u16, "".into()).serialize().unwrap()).unwrap();
            return;
        }
        
        let mut hasher = Groestl256::default();
        hasher.update(password);
        let hash = format!("{:x}", hasher.finalize());

        let entry = self.db.iter().find(|user_entry| user_entry.name == username).unwrap();
        if entry.password_hash != hash {
            stream.write_all(&AtlasPacket::LoginResponse(AtlasResponseCodes::IncorrectPassword as u16, "".into()).serialize().unwrap()).unwrap();
            return;
        }

        let token = USER_HANDLER.lock().unwrap().add_user(stream, entry);

        stream.write_all(&AtlasPacket::LoginResponse(AtlasResponseCodes::Success as u16, token).serialize().unwrap()).unwrap();
    }
}