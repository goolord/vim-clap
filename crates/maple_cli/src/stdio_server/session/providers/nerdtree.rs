use std::path::{self, Path, PathBuf};
use std::{fs, io};

use anyhow::Result;
use crossbeam_channel::Sender;
use log::debug;
use serde_json::json;

use icon::prepend_filer_icon;
use nerdtree::PathNode;

use crate::stdio_server::{
    session::{
        build_abs_path, Event, EventHandler, NewSession, OnMove, OnMoveHandler, Session,
        SessionContext, SessionEvent,
    },
    write_response, Message,
};

#[derive(Clone)]
pub struct ExplorerMessageHandler {
    pub tree_explorer: nerdtree::TreeExplorer,
}

impl EventHandler for ExplorerMessageHandler {
    fn handle(&mut self, event: Event, context: &SessionContext) {
        match event {
            Event::Toggle(msg) => {
                debug!("-------------- handle toggle message");
                let lnum = msg.get_lnum();
                let lines = self.tree_explorer.do_toggle(lnum - 1);

                let result = json!({
                "lines": lines,
                });

                let result = json!({ "id": msg.id, "provider_id": "nerdtree", "result": result });

                write_response(result);
            }
            _ => {}
        }
    }
}

pub struct ExplorerSession;

impl NewSession for ExplorerSession {
    fn spawn(&self, msg: Message) -> Result<Sender<SessionEvent>> {
        let (session_sender, session_receiver) = crossbeam_channel::unbounded();

        let cwd = msg.get_cwd();
        let lnum = msg.get_lnum();
        debug!("Recv nerdtree params: cwd:{}", cwd,);

        let root_node = PathNode::new(&cwd);

        handle_nerdtree_message(msg.clone());

        let session = Session {
            session_id: msg.session_id,
            context: msg.into(),
            event_handler: ExplorerMessageHandler {
                tree_explorer: nerdtree::TreeExplorer::new_simple(root_node),
            },
            event_recv: session_receiver,
        };

        session.start_event_loop()?;

        Ok(session_sender)
    }
}

pub fn handle_nerdtree_message(msg: Message) {
    tokio::spawn(async move {
        let cwd = msg.get_cwd();
        let lnum = msg.get_lnum();
        debug!("Recv nerdtree params: cwd:{}", cwd,);

        let mut root = PathNode::new(&cwd);
        let lines = root.expand_at(lnum - 1);

        let result = json!({
        "lines": lines,
        });

        let result = json!({ "id": msg.id, "provider_id": "nerdtree", "result": result });

        write_response(result);
    });
}

pub fn toggle(msg: Message) {
    tokio::spawn(async move {
        let cwd = msg.get_cwd();
        debug!("Recv nerdtree params: cwd:{}", cwd,);
        let lnum = msg.get_lnum();

        let mut root = PathNode::new_expanded(&cwd);

        let lines = root.expand_at(lnum - 1);

        let result = json!({
        "lines": lines,
        });

        let result = json!({ "id": msg.id, "provider_id": "nerdtree", "result": result });

        write_response(result);
    });
}

/*
pub struct TreeExplorerSession;

impl NewSession for TreeExplorerSession {
    fn new_session(&self, msg: Message, event_handler: T) -> Result<Sender<SessionEvent>> {
        let (session_sender, session_receiver) = crossbeam_channel::unbounded();

        let session = Session {
            session_id: msg.session_id,
            context: msg.into(),
            event_handler,
            event_recv: session_receiver,
        };

        debug!("new tree explorer session context: {:?}", session.context);

        session.start_event_loop()?;

        Ok(session_sender)
    }
}

#[derive(Clone)]
pub struct TreeExplorerEventHandler;

impl EventHandler for TreeExplorerEventHandler {
    fn handle(&self, event: Event, context: &SessionContext) {
        match event {
            Event::OnMove(msg) => {
                todo!("unimplemented for tree explorer")
            }
            Event::OnTyped(msg) => todo!("OnTyped unimplemented for tree explorer"),
        }
    }
}
*/
