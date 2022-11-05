use std::{
    io::{Read, Write},
    net::{SocketAddrV4, TcpStream, UdpSocket},
};

use crate::utils::{self, average, get_timestamp};

pub struct Client {
    server_addr: SocketAddrV4,
    is_udp: bool,
    latencies: Vec<u128>,
    count: usize,
}

impl Client {
    pub fn new(ip: String, port: usize, is_udp: bool, count: usize) -> Self {
        Self {
            server_addr: SocketAddrV4::new(ip.parse().unwrap(), port as u16),
            is_udp,
            latencies: vec![0; count],
            count,
        }
    }

    pub fn run(&mut self) {
        if self.is_udp {
            self.run_udp()
        } else {
            self.run_tcp()
        }
    }

    fn run_tcp(&mut self) {
        let mut stream = TcpStream::connect(self.server_addr).expect("Conn failed");

        let mut timestamp_buffer = [0; 16];
        for i in 0..self.count {
            {
                // send
                utils::format_ts(&mut timestamp_buffer, utils::get_timestamp());
                stream.write_all(&timestamp_buffer).expect("Conn broke");
            }

            self.latencies[i] = {
                // recv
                stream
                    .read_exact(&mut timestamp_buffer)
                    .expect("Conn broke");
                let recv_ts = utils::parse_ts(timestamp_buffer);

                (get_timestamp() - recv_ts) / 2
            }
        }
        self.print_result();
    }

    fn run_udp(&mut self) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Creating udp socket");

        let mut buf = [0; 16];
        for i in 0..self.count {
            let mut ok = false;
            while !ok {
                ok = {
                    // send
                    utils::format_ts(&mut buf, utils::get_timestamp());
                    16 == socket.send_to(&buf, self.server_addr).unwrap_or(0)
                };
                if !ok {
                    continue;
                }
                ok = {
                    // recv
                    socket
                        .recv_from(&mut buf)
                        .map(|(amt, src)| {
                            assert_eq!(src, std::net::SocketAddr::V4(self.server_addr));
                            if amt == 16 {
                                let recv_ts = utils::parse_ts(buf);
                                self.latencies[i] = (get_timestamp() - recv_ts) / 2;
                                true
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false)
                };
            }
        }
        self.print_result();
    }

    fn print_result(&self) {
        println!(
            "Test finished with avg rtt of {} us",
            average(&self.latencies)
        );
    }
}
