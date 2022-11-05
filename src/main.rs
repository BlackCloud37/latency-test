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
                    arg!(-p --port <PORT> "server port")
                        .value_parser(value_parser!(usize))
                        .default_value("65432"),
                )
                .arg(arg!(-u --udp "if this arg is set, use udp, else tcp")),
        )
        .subcommand(
            Command::new("client")
                .about("Start a test client")
                .arg(
                    arg!(-p --port <PORT> "server port")
                        .value_parser(value_parser!(usize))
                        .default_value("65432"),
                )
                .arg(arg!(-u --udp "if this arg is set, use udp, else tcp"))
                .arg(
                    arg!(-c --count <COUNT> "test how many times")
                        .value_parser(value_parser!(usize))
                        .default_value("100"),
                )
                .arg(arg!(<SERVER_IP> "server's ip")),
        )
}
mod client;
mod server;
mod utils;

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("server", sub_matches)) => {
            let port = *sub_matches.get_one::<usize>("port").unwrap();
            let is_udp = *sub_matches.get_one::<bool>("udp").unwrap();
            println!(
                "Start server at {} with mode {}",
                port,
                if is_udp { "UDP" } else { "TCP" }
            );
            let server = server::Server::new(port, is_udp);
            server.run();
        }
        Some(("client", sub_matches)) => {
            let port = *sub_matches.get_one::<usize>("port").unwrap();
            let server_ip = sub_matches
                .get_one::<String>("SERVER_IP")
                .expect("required");
            let is_udp = *sub_matches.get_one::<bool>("udp").unwrap();
            let count = *sub_matches.get_one::<usize>("count").unwrap();
            println!(
                "Start client to {}:{} with mode {}",
                server_ip,
                port,
                if is_udp { "UDP" } else { "TCP" }
            );
            let mut client = client::Client::new(server_ip.into(), port, is_udp, count);
            client.run();
        }
        _ => unreachable!(),
    }
}
