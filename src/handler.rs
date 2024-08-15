use std::net::TcpStream;

use atlas_shared::packet::AtlasPacket;

use crate::{CHAT_HANDLER, USER_DB};

pub struct PacketHandler;

impl PacketHandler {
    pub fn handle_packet(stream: &mut TcpStream, packet: AtlasPacket) {
        match packet {
            AtlasPacket::RegisterRequest(username, password) => USER_DB.lock().unwrap().register(stream, username, password),
            AtlasPacket::LoginRequest(username, password, _) => USER_DB.lock().unwrap().login(stream, username, password),
            AtlasPacket::JoinChatroomRequest(room, token) => CHAT_HANDLER.lock().unwrap().join_room(stream, token, room),
            AtlasPacket::SendMessageRequest(message, token) => CHAT_HANDLER.lock().unwrap().send_message(stream, token, message),
            _ => todo!("impl packet {:?}", packet),
        }
    }
}