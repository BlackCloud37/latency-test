use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UdpSocket},
};

pub struct Server {
    pub tport: usize,
    pub uport: usize,
}

impl Server {
    pub async fn run(self) {
        let uaddr = format!("0.0.0.0:{}", self.uport);
        let u = tokio::spawn(async move {
            let socket = UdpSocket::bind(&uaddr)
                .await
                .expect("error creating udp socket");
            println!("[UDP] listen on {}", uaddr);
            let mut buf = vec![0; 1024];
            let mut to_send = None;
            loop {
                if let Some((size, peer)) = to_send {
                    socket.send_to(&buf[..size], &peer).await.unwrap_or(0);
                }

                to_send = socket
                    .recv_from(&mut buf)
                    .await
                    .map(|v| Some(v))
                    .unwrap_or(None);
            }
        });

        let taddr = format!("0.0.0.0:{}", self.tport);
        let t = tokio::spawn(async move {
            let listener = TcpListener::bind(&taddr)
                .await
                .expect("error creating tcp listener");
            println!("[TCP] listen on {}", taddr);

            loop {
                if let Ok((mut socket, src)) = listener.accept().await {
                    println!("[TCP] connect from {}", src);
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
        let _ = tokio::join!(u, t); // block forever
        unreachable!()
    }
}
