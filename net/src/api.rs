use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::net::SocketAddr;

use data::{Config, ListenerFn, NetHandler, SendClosureFn, Socket};
use net::{handle_bind, handle_close, handle_connect, handle_net, handle_send};

pub struct NetManager {
    net_sender: Sender<SendClosureFn>,
}

impl NetManager {
    /// call by logic thread
    pub fn new() -> Self {
        let (s, r) = mpsc::channel::<SendClosureFn>();
        let net_sender = s.clone();

        // create net thread
        thread::spawn(move || {
            handle_net(s, r);
        });

        Self { net_sender }
    }

    /// call by logic thread
    pub fn bind(&self, addr: SocketAddr, config: Config, func: ListenerFn) {
        let data = Box::new(move |handler: &mut NetHandler| {
            handle_bind(handler, addr, config, func);
        });

        self.net_sender.send(data).unwrap();
    }

    /// call by logic thread
    pub fn connect(&self, config: Config, func: ListenerFn) {
        let data = Box::new(move |handler: &mut NetHandler| {
            handle_connect(handler, config, func);
        });

        self.net_sender.send(data).unwrap();
    }
}

impl Socket {
    /// call by logic thread
    pub fn send(&self, buf: Arc<Vec<u8>>) {
        let socket = self.socket;
        let data = Box::new(move |handler: &mut NetHandler| {
            handle_send(handler, socket, buf);
        });

        self.sender.send(data).unwrap();
    }

    /// call by logic thread
    pub fn close(&self, force: bool) {
        let socket = self.socket;
        let data = Box::new(move |handler: &mut NetHandler| {
            handle_close(handler, socket, force);
        });

        self.sender.send(data).unwrap();
    }
}
