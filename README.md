# Test Network Latency

Client and server for network latency testing(via TCP/UDP).

## Usage

For testing network latency between host A and B, you should setup a test server first (for example, at host A), then use client in host B to ping A and get the result.

The binary supports both server and client mode, usage:
```
# latency-test -h
Test latency via TCP/UDP

Usage: latency-test <COMMAND>

Commands:
  server  Start a test server
  client  Start a test client
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

### Server
```
# latency-test server -h
Start a test server

Usage: latency-test server [OPTIONS]

Options:
  -t, --tport <TPORT>  tcp port [default: 65432]
  -u, --uport <UPORT>  udp port [default: 65433]
  -h, --help           Print help information
```

### Client
```
# latency-test client -h
Start a test client

Usage: latency-test client [OPTIONS] <SERVER_IP>

Arguments:
  <SERVER_IP>  server's ip

Options:
  -p, --port <PORT>          server port [default: 0]
  -u, --udp                  if this arg is set, use udp, else tcp
  -c, --count <COUNT>        test how many times [default: 100]
  -d, --dup <DUP>            dup packet count [default: 1]
  -s, --size <SIZE>          size of payload in bytes, max is 1024(B) [default: 256]
  -i, --interval <INTERVAL>  interval between each packet in ms [default: 0]
  -q, --quiet                if this flag is set, only print final result
  -h, --help                 Print help information
```

### Example
1. Start the test server at host A with default options
    ```
    Host-A $ latency-test server
    Start server at 65432(TCP) 65433(UDP)
    [UDP] listen on 0.0.0.0:65433
    [TCP] listen on 0.0.0.0:65432
    ```
2. Start an TCP test client at host B, test 10 rounds (-c 10)
    ```
    Host-B $ latency-teslatency-test client <SERVER_IP> 1 -c 10 
    Start client Client { server_addr: <SERVER_IP>:65432, is_udp: false, count: 10, dup: 1, size: 256, interval: 0, quiet: false }
    [TCP] pkt 0 received with latency 35397us
    [TCP] pkt 1 received with latency 41850us
    [TCP] pkt 2 received with latency 35318us
    [TCP] pkt 3 received with latency 35937us
    [TCP] pkt 4 received with latency 35298us
    [TCP] pkt 5 received with latency 35279us
    [TCP] pkt 6 received with latency 35284us
    [TCP] pkt 7 received with latency 35288us
    [TCP] pkt 8 received with latency 35298us
    [TCP] pkt 9 received with latency 35303us
    Result latency(RTT/2) in microsecs: AVG(36025.2) MIN(35279) MAX(41850)
    ```
3. Start an UDP test client at host B, test 10 rounds (-c 10)
    ```
    Host-B $ latency-teslatency-test client <SERVER_IP> 1 -c 10 -u
    Start client Client { server_addr: <SERVER_IP>:65433, is_udp: true, count: 10, dup: 1, size: 256, interval: 0, quiet: false }
    [UDP] pkt 0 received with latency 35174us
    [UDP] pkt 1 received with latency 35121us
    [UDP] pkt 2 received with latency 35126us
    [UDP] pkt 3 received with latency 35285us
    [UDP] pkt 4 received with latency 35112us
    [UDP] pkt 5 received with latency 35114us
    [UDP] pkt 6 received with latency 35097us
    [UDP] pkt 7 received with latency 35096us
    [UDP] pkt 8 received with latency 35099us
    [UDP] pkt 9 received with latency 35104us
    Result latency(RTT/2) in microsecs: AVG(35132.8) MIN(35096) MAX(35285)
    ```