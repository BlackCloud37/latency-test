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
                    arg!(-d --dup <DUP> "dup packet count")
                        .value_parser(value_parser!(usize))
                        .default_value("1"),
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
                    arg!(--interface <INTERFACE> "interface to use")
                )
                .arg(arg!(-q --quiet "if this flag is set, only print final result"))
                .arg(arg!(<SERVER_IP> "server's ip")),
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
            println!("Start server at {}(TCP) {}(UDP)", tport, uport);
            let server = server::Server { tport, uport };
            server.run().await;
        }
        Some(("client", sub_matches)) => {
            let port = *sub_matches.get_one::<usize>("port").unwrap();
            let server_ip = sub_matches
                .get_one::<String>("SERVER_IP")
                .expect("required");
            let is_udp = *sub_matches.get_one::<bool>("udp").unwrap();
            let quiet = *sub_matches.get_one::<bool>("quiet").unwrap();
            let count = *sub_matches.get_one::<usize>("count").unwrap();
            let dup = *sub_matches.get_one::<usize>("dup").unwrap();
            let size = *sub_matches.get_one::<usize>("size").unwrap();
            let interval = *sub_matches.get_one::<usize>("interval").unwrap();
            let interface = sub_matches.get_one::<String>("interface").map(|iface| iface.to_string());
            if dup > 1 && !is_udp {
                println!("[WARNING] dup is ignored in TCP mode");
            }
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

            let mut client = client::Client {
                count,
                dup,
                is_udp,
                size,
                interval,
                quiet,
                interface,
                server_addr: format!("{}:{}", server_ip, server_port).parse().unwrap(),
            };
            if !quiet {
                println!("Start client {:?}", client);
            }

            client.run().await;
        }
        _ => unreachable!(),
    }
}
