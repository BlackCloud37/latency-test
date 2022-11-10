use std::{net::SocketAddr, time::Duration};

use tokio::{net::{TcpStream, TcpSocket, UdpSocket}, io::{AsyncWriteExt, AsyncReadExt}, time::{self, Instant}};

use crate::utils::{average, random_buffer};

#[derive(Debug)]
pub struct Client {
    pub server_addr: SocketAddr,
    pub is_udp: bool,
    pub count: usize,
    pub dup: usize,
    pub size: usize,
    pub interval: usize,
    pub quiet: bool,
    pub interface: Option<String>
}

impl Client {
    pub async fn run(&mut self) {
        if self.is_udp {
            self.run_udp().await;
        } else {
            self.run_tcp().await;
        }
    }

    async fn run_tcp(&mut self) {
        let mut latencies = vec![0; self.count];
        
        let mut stream = match &self.interface {
            Some(iface) => {
                let socket = TcpSocket::new_v4().expect("create socket");
                socket.bind_device(Some(iface.as_bytes())).expect("bind to interface");
                socket.connect(self.server_addr).await.expect("connect")
            },
            None => {
                TcpStream::connect(self.server_addr).await.expect("connect")
            }   
        };
        stream.set_nodelay(true).expect("set nodelay");

        let mut buf = vec![0; self.size];
        let interval = Duration::from_millis(self.interval as u64);
        for i in 0..self.count {
            // send
            random_buffer(&mut buf);

            let begin = Instant::now();
            stream.write_all(&buf).await.expect("conn broken");
            stream.read_exact(&mut buf).await.expect("conn broken");

            latencies[i] = begin.elapsed().as_micros() / 2;
            if self.interval > 0 {
                std::thread::sleep(interval);
            }
            if !self.quiet {
                println!("[TCP] pkt {} received with latency {}us", i, latencies[i]);
            }
        }
        Self::print_result(&latencies);
    }

    async fn run_udp(&mut self) {
        let mut latencies = vec![0; self.count];
        let socket = UdpSocket::bind("0.0.0.0:0").await.expect("creating udp socket");
        if let Some(iface) = &self.interface {
            socket.bind_device(Some(iface.as_bytes())).expect("bind interface");
        };

        socket
            .connect(self.server_addr)
            .await
            .expect("connect function failed");

        let mut buf = vec![0; self.size];
        let interval = Duration::from_millis(self.interval as u64);
        for i in 0..self.count {
            random_buffer(&mut buf);

            while latencies[i] == 0 {
                let begin = Instant::now();
                for _ in 0..self.dup {
                    socket.send(&buf).await.expect("could not send message");
                }

                for _ in 0..self.dup {
                    match time::timeout(Duration::from_secs(3), socket.recv(&mut buf)).await {
                        Ok(res) => {
                            let amt = res.expect("recv error");
                            assert_eq!(amt, self.size);
                            if latencies[i] == 0 {
                                latencies[i] = begin.elapsed().as_micros() / 2;
                            }       
                        },
                        Err(_) => {
                            // timeout
                            continue;
                        },
                    }
                }
            }
            if self.interval > 0 {
                tokio::time::sleep(interval).await;
            }
            if !self.quiet {
                println!("[UDP] pkt {} received with latency {}us", i, latencies[i]);
            }
        }
        Self::print_result(&latencies);
    }

    fn print_result(latencies: &[u128]) {
        println!(
            "Result latency(RTT/2) in microsecs: AVG({}) MIN({}) MAX({})",
            average(&latencies),
            latencies.iter().min().unwrap(),
            latencies.iter().max().unwrap(),
        );
    }
}
