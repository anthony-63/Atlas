use std::{io::Write, net::TcpStream};

use atlas_shared::packet::{AtlasPacket, AtlasResponseCodes};

use crate::{user::User, USER_HANDLER};

pub struct ChatHandler {
    pub rooms: Vec<String>,
}

impl ChatHandler {
    pub fn join_room(&mut self, stream: &mut TcpStream, token: String, room: String) {
        if !USER_HANDLER.lock().unwrap().is_online(token.clone()) {
            stream.write_all(&AtlasPacket::JoinChatroomResponse(AtlasResponseCodes::IncorrectToken as u16).serialize().unwrap()).unwrap();
            return;
        }
        
        if !USER_HANDLER.lock().unwrap().set_room(token, room) {
            stream.write_all(&AtlasPacket::JoinChatroomResponse(AtlasResponseCodes::ChatroomDoesntExist as u16).serialize().unwrap()).unwrap();
            return;
        }

        stream.write_all(&AtlasPacket::JoinChatroomResponse(AtlasResponseCodes::Success as u16).serialize().unwrap()).unwrap();
    }

    pub fn send_message(&mut self, stream: &mut TcpStream, token: String, message: String) {
        let mut locked_user_handler = USER_HANDLER.lock();
        let user_handler = locked_user_handler.as_mut().unwrap();
        
        if !user_handler.is_online(token.clone()) {
            stream.write_all(&AtlasPacket::SendMessageResponse(AtlasResponseCodes::IncorrectToken as u16).serialize().unwrap()).unwrap();
            return;
        }
        
        let user_index = user_handler.get_user_index(token.clone());
        let username = &user_handler.online[user_index].user.name.clone();
        let user_room = &user_handler.online[user_index].joined_room.clone();

        drop(locked_user_handler);

        for user in USER_HANDLER.lock().unwrap().online.iter_mut() {
            if user.joined_room != *user_room || token == user.token {
                continue;
            }
            user.stream.write_all(&AtlasPacket::RecvMessage(username.clone(), message.clone()).serialize().unwrap()).unwrap();
        }
        stream.write_all(&AtlasPacket::SendMessageResponse(AtlasResponseCodes::Success as u16).serialize().unwrap()).unwrap();
    }
}