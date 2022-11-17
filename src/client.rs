use std::{net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream, UdpSocket},
    sync::{Barrier, Semaphore},
    time::{self, Instant},
};

use crate::utils::random_buffer;

#[derive(Debug)]
pub struct Client {
    pub is_udp: bool,
    pub count: usize,
    pub size: usize,
    pub interval: usize,
    pub conns: Vec<(Option<String>, SocketAddr, u64)>, // Option(iface), dst, id
    pub local_port: usize,
}

impl Client {
    pub async fn run(&mut self) {
        if self.is_udp {
            self.run_udp().await;
        } else {
            self.run_tcp().await;
        }
    }

    async fn run_tcp(&self) {
        let mut handles = Vec::with_capacity(self.conns.len());
        let barrier = Arc::new(Barrier::new(self.conns.len()));
        let sem = Arc::new(Semaphore::new(1)); // only 1 on-fly packet is allowed

        for (iface, dst, id) in self.conns.clone().into_iter() {
            let interval = Duration::from_millis(self.interval as u64);
            let size = self.size;
            let count = self.count;
            let wall = barrier.clone();
            let sem = sem.clone();
            let iface_str = if let Some(iface) = &iface {
                iface.to_owned()
            } else {
                "default".to_string()
            };
            let local_port = self.local_port;
            handles.push(tokio::spawn(async move {
                // connect
                let mut stream = match iface {
                    Some(iface) => {
                        let socket = TcpSocket::new_v4().expect("create socket");
                        socket.bind(format!("0.0.0.0:{}", local_port).parse().unwrap()).expect("bind");
                        socket
                            .bind_device(Some(iface.as_bytes()))
                            .expect("bind to interface");
                        socket.connect(dst).await.expect("connect")
                    }
                    _ => TcpStream::connect(dst).await.expect("connect"),
                };
                stream.set_nodelay(true).expect("set nodelay");
                wall.wait().await;

                // send/recv loop
                let mut buf = vec![0; size];
                random_buffer(&mut buf);
                for i in 0..count {
                    let lat = {
                        // get permission to send/recv one pkt, all other tasks will wait
                        let _permit = sem.acquire().await.expect("error getting semaphore");

                        // send
                        let begin = Instant::now();
                        stream.write_all(&buf).await.expect("conn broken");

                        // recv
                        stream.read_exact(&mut buf).await.expect("conn broken");

                        begin.elapsed().as_micros() / 2
                    };

                    println!(
                        "[TCP]({},{},{}) pkt {} received with latency {}us",
                        iface_str, dst, id, i, lat
                    );
                    tokio::time::sleep(interval).await;

                    wall.wait().await;
                }
            }));
        }
        futures::future::join_all(handles).await;
    }

    async fn run_udp(&self) {
        let mut handles = Vec::with_capacity(self.conns.len());
        let barrier = Arc::new(Barrier::new(self.conns.len()));
        let sem = Arc::new(Semaphore::new(1)); // only 1 on-fly packet is allowed

        for (iface, dst, id) in self.conns.clone().into_iter() {
            let interval = Duration::from_millis(self.interval as u64);
            let size = self.size;
            let count = self.count;
            let wall = barrier.clone();
            let sem = sem.clone();
            let iface_str = if let Some(iface) = &iface {
                iface.to_owned()
            } else {
                "default".to_string()
            };
            let local_port = self.local_port;
            handles.push(tokio::spawn(async move {
                let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port))
                    .await
                    .expect("creating udp socket");
                if let Some(iface) = iface {
                    socket
                        .bind_device(Some(iface.as_bytes()))
                        .expect("bind interface");
                };
                socket.connect(dst).await.expect("connect function failed");
                wall.wait().await;

                // send/recv loop
                let mut buf = vec![0; size];
                random_buffer(&mut buf);

                for i in 0..count {
                    let lat = {
                        let _permit = sem.acquire().await.expect("error getting semaphore");

                        let begin = Instant::now();
                        socket.send(&buf).await.expect("could not send message");

                        match time::timeout(Duration::from_secs(3), socket.recv(&mut buf)).await {
                            Ok(res) => {
                                let amt = res.expect("recv error");
                                assert_eq!(amt, size);
                                begin.elapsed().as_micros() / 2
                            }
                            Err(_) => {
                                // timeout
                                0
                            }
                        }
                    };

                    if lat != 0 {
                        println!(
                            "[UDP]({},{},{}) pkt {} received with latency {}us",
                            iface_str, dst, id, i, lat
                        );
                    } else {
                        println!("[UDP]({},{},{}) pkt {} loss", iface_str, dst, id, i);
                    }
                    tokio::time::sleep(interval).await;
                    wall.wait().await;
                }
            }));
        }
        futures::future::join_all(handles).await;
    }
}
