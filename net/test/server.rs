use std::io::Result;
use std::ops::Range;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use net::{Config, NetManager, Protocol, Socket, Stream};

fn handle_close(stream_id: usize, reason: Result<()>) {
    println!(
        "server handle_close, stream_id = {}, reason = {:?}",
        stream_id, reason
    );
}

fn handle_recv(socket: Socket, stream: Arc<RwLock<Stream>>, begin: usize, end: usize) {
    let s = stream.clone();
    println!("server, request recv [{}, {}]", begin, end);

    let func = Box::new(move |data: Result<Arc<Vec<u8>>>| {
        {
            let s_borrow = &s.read().unwrap();

            let b = data.unwrap();
            for (i, &d) in b.iter().enumerate() {
                assert_eq!((begin + i) as u8, d);
            }

            let mut buf: Vec<u8> = vec![];
            for i in begin..end {
                buf.push(i as u8);
            }
            socket.send(Arc::new(buf));

            println!("server recv, valid, begin = {}, end = {}", begin, end);
        }

        if end == 1024 * 1024 {
            socket.close(true);
            return;
        }

        let mut new_end = end + end - begin;
        if new_end > 1024 * 1024 {
            new_end = 1024 * 1024;
        }
        handle_recv(socket, s, end, new_end);
    });

    let r = stream.write().unwrap().recv(end - begin, func);
    if let Some((func, data)) = r {
        func(data);
    }
}

fn handle_bind(peer: Result<(Socket, Arc<RwLock<Stream>>)>, addr: Result<SocketAddr>) {
    println!("server handle_bind: addr = {:?}", addr.unwrap());

    let (socket, stream) = peer.unwrap();

    {
        let s = &mut stream.write().unwrap();

        s.set_close_callback(Box::new(|id, reason| handle_close(id, reason)));
        s.set_send_buf_size(1024 * 1024);
        s.set_recv_timeout(5 * 1000);
    }

    handle_recv(socket, stream, 0, 1 * 1024);
}

pub fn start_server() -> NetManager {
    let mgr = NetManager::new();
    let config = Config {
        protocol: Protocol::TCP,
        server_addr: None,
    };

    let addr = "127.0.0.1:1234".parse().unwrap();
    mgr.bind(addr, config, Box::new(|peer, addr| handle_bind(peer, addr)));
    return mgr;
}
