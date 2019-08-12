use std::{
    io,
    net::{SocketAddr, UdpSocket},
    str,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread, time,
};
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let opt = Cli::from_args();

    let address: SocketAddr = opt.remote.parse().expect("!! Unable to parse socket address");

    let stdin = stdin_thread();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", opt.port))?;
    socket.set_nonblocking(true)?;
    let mut buf = [0; 255];
    socket.send_to("Client connected!\n".as_bytes(), address)?;

    let port = socket.local_addr().expect("!! Unable to get local socket address").port();

    println!("Welcome to PunchChat!");
    println!("We are listening on port {}.", port);

    loop {
        // Socket send
        match stdin.try_recv() {
            Ok(msg) => {
                socket.send_to(msg.as_bytes(), address)?;
                print!(">> {}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("!! Stdin channel disconnected"),
        }

        // Socket receive
        let (amt, _) = match socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => panic!("!! Cannot receive: {}", e),
        };

        let buf = &mut buf[..amt];

        let s = match str::from_utf8(buf) {
            Ok(v) => v,
            Err(e) => panic!("!! Invalid UTF-8 sequence: {}", e),
        };

        print!("<< {}", s);
        thread::sleep(time::Duration::from_millis(50));
    }
}

fn stdin_thread() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        if buffer.len() > 255 {
            println!("!! Message to send is too long,");
            continue;
        }

        tx.send(buffer).unwrap();
    });

    rx
}

/// Chat with another client, serverless through most NAT's.
#[derive(StructOpt)]
pub struct Cli {
    /// The remote IP to connect to.
    remote: String,
    /// The port to expose.
    /// If this is not set a random port will be assigned.
    #[structopt(short = "p", long = "port", default_value = "0")]
    port: u16,
}
