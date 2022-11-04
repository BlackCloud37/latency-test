use std::{net::{TcpListener, TcpStream, UdpSocket}, io::{Read, Write}};

use crate::utils;

pub struct Server {
    port: usize,
    is_udp: bool,
}

impl Server {
    pub fn new(port: usize, is_udp: bool) -> Self {
        Self { port, is_udp }
    }

    pub fn run(&self) -> ! {
        // TODO: bind core
        if self.is_udp {
            self.run_udp()
        } else {
            self.run_tcp()
        }
    }

    fn run_udp(&self) -> ! {
        let addr = format!("127.0.0.1:{}", self.port);
        let socket = UdpSocket::bind(addr).expect("Error creating socket");

        let mut buf = [0; 16];

        loop {
            if let Ok((amt, src)) = socket.recv_from(&mut buf) {
                assert_eq!(amt, 16);
                let time_diff = {
                    let recv_ts = utils::parse_ts(buf);
                    utils::get_timestamp() - recv_ts
                };
                utils::format_ts(&mut buf, utils::get_timestamp() - time_diff);
                socket.send_to(&buf, &src).ok();
            }
        }
    }

    fn run_tcp(&self) -> ! {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(addr).expect("Error listening");
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_tcp_conn(stream);                    
                },
                Err(e) => {
                    println!("Error listening {}", e)
                }
            }
        }
        unreachable!()
    }

    fn handle_tcp_conn(&self, mut stream: TcpStream) {
        stream.set_nodelay(true).unwrap();
        
        let mut timestamp_buffer = [0; 16];
        while let Ok(()) = stream.read_exact(&mut timestamp_buffer) {
            let time_diff = {
                let recv_ts = utils::parse_ts(timestamp_buffer);
                utils::get_timestamp() - recv_ts
            };
            
            utils::format_ts(&mut timestamp_buffer, utils::get_timestamp() - time_diff);
            if let Err(_) = stream.write_all(&timestamp_buffer) {
                break;
            }
        }
        println!("Conn from {:?} closed", stream.peer_addr());
    }
}