use std::{fs::File, net::SocketAddr};

use clap::{arg, value_parser, Command};

fn cli() -> Command {
    Command::new("latency")
        .about("Test latency via TCP/UDP")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("server")
                .about("Start a test server")
                .arg(
                    arg!(-t --tport <TPORT> "tcp port")
                        .value_parser(value_parser!(usize))
                        .default_value("65432"),
                )
                .arg(
                    arg!(-u --uport <UPORT> "udp port")
                        .value_parser(value_parser!(usize))
                        .default_value("65433"),
                )
                .arg(
                    arg!(--interface <INTERFACE> "interface to use")
                ),
        )
        .subcommand(
            Command::new("client")
                .about("Start a test client")
                .arg(
                    arg!(-p --port <PORT> "server port")
                        .value_parser(value_parser!(usize))
                        .default_value("0"),
                )
                .arg(arg!(-u --udp "if this arg is set, use udp, else tcp"))
                .arg(
                    arg!(-c --count <COUNT> "test how many times")
                        .value_parser(value_parser!(usize))
                        .default_value("100"),
                )
                .arg(
                    arg!(-s --size <SIZE> "size of payload in bytes, max is 1024(B)")
                        .value_parser(value_parser!(usize))
                        .default_value("256"),
                )
                .arg(
                    arg!(-i --interval <INTERVAL> "interval between each packet in ms")
                        .value_parser(value_parser!(usize))
                        .default_value("0"),
                )
                .arg(
                    arg!(--interface <INTERFACE> "interface to use, will be override by config")
                )
                .arg(
                    arg!(-C --connections <CONNECTIONS> "config file for multi connections")                )
                .arg(arg!(-d --dest <SERVER_IP> "server's ip")),
        )
}
mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("server", sub_matches)) => {
            let tport = *sub_matches.get_one::<usize>("tport").unwrap();
            let uport = *sub_matches.get_one::<usize>("uport").unwrap();
            let interface = sub_matches.get_one::<String>("interface").map(|iface| iface.to_string());
            println!("Start server at {}(TCP) {}(UDP)", tport, uport);
            let server = server::Server { tport, uport, interface };
            server.run().await;
        }
        Some(("client", sub_matches)) => {
            let port = *sub_matches.get_one::<usize>("port").unwrap();
            let server_ip = sub_matches.get_one::<String>("dest");
            let is_udp = *sub_matches.get_one::<bool>("udp").unwrap();
            let count = *sub_matches.get_one::<usize>("count").unwrap();
            let size = *sub_matches.get_one::<usize>("size").unwrap();
            let interval = *sub_matches.get_one::<usize>("interval").unwrap();
            let interface = sub_matches.get_one::<String>("interface").map(|iface| iface.to_string());
            assert!(count >= 1);
            assert!(size >= 1 && size <= 1024);

            let server_port = if port == 0 {
                if is_udp {
                    65433
                } else {
                    65432
                }
            } else {
                port
            };
            let conn_config = sub_matches.get_one::<String>("connections").map(|f| f.to_string());
            let conns = if let Some(conn_config) = conn_config {
                if interface.is_some() {
                    println!("[WARN] --interface will be override by --connections");
                }
                if server_ip.is_some() {
                    println!("[WARN] SERVER_IP will be override by --connections");
                }
                
                let conn_config_file = File::open(conn_config).expect("open conn file");
                let v: serde_json::Value = serde_json::from_reader(conn_config_file).expect("read json file");
                let mut conns = vec![];
                for (iface, conf) in v.as_object().expect("parse json") {
                    for (dst, cnt) in conf.as_object().expect("parse json") {
                        for i in 0..cnt.as_u64().expect("parse json") {
                            conns.push((Some(iface.to_owned()), dst.to_owned().parse::<SocketAddr>().unwrap(), i));
                        }
                    }
                }
                conns
            } else {
                vec![(interface, format!("{}:{}", server_ip.expect("required"), server_port).parse().unwrap(), 0)]
            };

            let mut client = client::Client {
                count,
                is_udp,
                size,
                interval,
                conns
            };
            client.run().await;
        }
        _ => unreachable!(),
    }
}
