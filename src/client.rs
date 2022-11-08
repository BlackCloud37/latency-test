use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddrV4, TcpStream, UdpSocket},
    time::{Duration, Instant},
};

use crate::utils::{average, random_buffer};

#[derive(Debug)]
pub struct Client {
    pub server_addr: SocketAddrV4,
    pub is_udp: bool,
    pub count: usize,
    pub dup: usize,
    pub size: usize,
    pub interval: usize,
    pub quiet: bool,
}

impl Client {
    pub fn run(&mut self) {
        if self.is_udp {
            self.run_udp()
        } else {
            self.run_tcp()
        }
    }

    fn run_tcp(&mut self) {
        let mut latencies = vec![0; self.count];

        let mut stream = TcpStream::connect(self.server_addr).expect("conn failed");
        stream.set_nodelay(true).expect("set nodelay");

        let mut buf = vec![0; self.size];
        let interval = Duration::from_millis(self.interval as u64);
        for i in 0..self.count {
            // send
            random_buffer(&mut buf);

            let begin = Instant::now();
            stream.write_all(&buf).expect("conn broken");
            stream.read_exact(&mut buf).expect("conn broken");

            latencies[i] = begin.elapsed().as_micros() / 2;
            if self.interval > 0 {
                std::thread::sleep(interval);
            }
            if !self.quiet {
                println!("[TCP] pkt {} received with rtt {}us", i, latencies[i]);
            }
        }
        Self::print_result(&latencies);
    }

    fn run_udp(&mut self) {
        let mut latencies = vec![0; self.count];

        let socket = UdpSocket::bind("0.0.0.0:0").expect("creating udp socket");
        socket
            .connect(self.server_addr)
            .expect("connect function failed");
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .expect("set read timeout");

        let mut buf = vec![0; self.size];
        let interval = Duration::from_millis(self.interval as u64);
        for i in 0..self.count {
            random_buffer(&mut buf);

            while latencies[i] == 0 {
                let begin = Instant::now();
                for _ in 0..self.dup {
                    socket.send(&buf).expect("could not send message");
                }

                for _ in 0..self.dup {
                    match socket.recv(&mut buf) {
                        Ok(amt) => {
                            assert_eq!(amt, self.size);
                            if latencies[i] == 0 {
                                latencies[i] = begin.elapsed().as_micros() / 2;
                            }
                        }
                        Err(e) => match e.kind() {
                            ErrorKind::WouldBlock | ErrorKind::TimedOut => continue,
                            _ => panic!("error receiving"),
                        },
                    }
                }
            }
            if self.interval > 0 {
                std::thread::sleep(interval);
            }
            if !self.quiet {
                println!("[UDP] pkt {} received with rtt {}us", i, latencies[i]);
            }
        }
        Self::print_result(&latencies);
    }

    fn print_result(latencies: &[u128]) {
        println!(
            "Result RTT/2 in microsecs: AVG({}) MIN({}) MAX({})",
            average(&latencies),
            latencies.iter().min().unwrap(),
            latencies.iter().max().unwrap(),
        );
    }
}
