use serde_json::json;

use crate::session::{HandleMessage, RpcMessage, SessionContext};
use crate::types::Message;
use crate::write_response;

pub mod on_init;
pub mod on_move;
pub mod on_typed;

#[derive(Clone)]
pub struct MessageHandler;

impl HandleMessage for MessageHandler {
    fn handle(&self, msg: RpcMessage, context: &SessionContext) {
        match msg {
            RpcMessage::OnMove(msg) => {
                let msg_id = msg.id;
                if let Err(e) = on_move::OnMoveHandler::try_new(&msg, context).map(|x| x.handle()) {
                    log::error!("Handle RpcMessage::OnMove {:?}, error: {:?}", msg, e);
                    write_response(json!({"error": format!("{}",e), "id": msg_id }));
                }
            }
            RpcMessage::OnTyped(msg) => on_typed::handle_on_typed(msg, context),
        }
    }
}
