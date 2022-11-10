use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UdpSocket, TcpSocket},
};

pub struct Server {
    pub tport: usize,
    pub uport: usize,
    pub interface: Option<String>
}

impl Server {
    pub async fn run(self) {
        let uaddr = format!("0.0.0.0:{}", self.uport);
        let uinterface = self.interface.clone();
        let udp_task = tokio::spawn(async move {
            let socket = UdpSocket::bind(&uaddr)
                .await
                .expect("error creating udp socket");
            if let Some(iface) = &uinterface {
                socket.bind_device(Some(iface.as_bytes())).expect("bind interface");
            }
            println!("[UDP] listen on {} with interface {:?}", uaddr, uinterface);
            
            let mut buf = vec![0; 1024];
            let mut to_send = None;
            loop {
                if let Some((size, peer)) = to_send {
                    socket.send_to(&buf[..size], &peer).await.unwrap_or(0);
                }

                to_send = socket
                    .recv_from(&mut buf)
                    .await
                    .ok();
            }
        });

        let taddr = format!("0.0.0.0:{}", self.tport);
        let tinterface = self.interface.clone();
        let tcp_task = tokio::spawn(async move {
            let listener = match &tinterface {
                Some(iface) => {
                    let socket = TcpSocket::new_v4().expect("create socket");
                    socket.bind_device(Some(iface.as_bytes())).expect("bind interface");
                    socket.set_reuseaddr(true).expect("set reuseaddr");
                    socket.bind(taddr.parse().unwrap()).expect("bind addr");
                    socket.listen(1024).expect("listen")
                }
                None => {
                    TcpListener::bind(&taddr)
                        .await
                        .expect("error creating tcp listener")
                }
            };
            
            println!("[TCP] listen on {} with interface {:?}", taddr, tinterface);

            loop {
                if let Ok((mut socket, src)) = listener.accept().await {
                    println!("[TCP] connect from {}", src);
                    socket.set_nodelay(true).expect("set nodelay");
                    tokio::spawn(async move {
                        let mut buf = vec![0; 1024];
                        loop {
                            let n = socket.read(&mut buf).await.unwrap_or(0);
                            if n == 0 {
                                break;
                            }

                            if socket.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                        }
                        println!("[TCP] disconnect from {}", src);
                    });
                }
            }
        });
        let _ = tokio::join!(udp_task, tcp_task); // block forever
        unreachable!()
    }
}
