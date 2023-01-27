use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;

fn main() {
    let mode = std::env::args().nth(1);
    match mode {
        Some(mode) => match mode.as_str() {
            "client" => client(),
            "server" => server(),
            mode => println!(
                "ERROR: Please specify either 'client' or 'server' (received '{}')",
                mode
            ),
        },
        None => println!("ERROR: Please specify either 'client' or 'server'"),
    }
}

// hostname and port for the server to bind to
const HOST: &str = "localhost:3333";

/// number of bytes for server to produce and client to read
const TOTAL_BYTES: usize = 1_000_000;

fn client() {
    // test with a raw TCP stream
    client_inner("TcpStream (Read)", TcpStream::connect(HOST).unwrap());

    // test with the stream wrapped in a buffered reader
    client_inner(
        "BufReader<TcpStream> (BufRead)",
        BufReader::new(TcpStream::connect(HOST).unwrap()),
    );
}

fn client_inner<TRead: Read>(description: &str, mut stream: TRead) {
    let mut index = 0;

    // create a large buffer to hold all incoming data
    let mut buffer: Vec<u8> = (0..TOTAL_BYTES).map(|_| 0).collect();

    let start = Instant::now();
    // loop while there are still bytes to be read
    loop {
        // get the next one-byte slice of the buffer to use for reading (the
        // tiny slice is chosen to make this as inefficient as possible, for
        // illustration purposes)
        let buffer_slice = &mut buffer[index..usize::min(index + 1, TOTAL_BYTES)];

        // read new data into the buffer slice
        let received_bytes = stream.read(buffer_slice).unwrap();

        if received_bytes > 0 {
            // advance `index` by the number of bytes read
            index += received_bytes;
        } else {
            // if there are no more bytes to be read, we're done
            break;
        }
    }
    let end = Instant::now();

    println!(
        "{} took {}ms",
        description,
        end.duration_since(start).as_millis()
    );
}

fn server() {
    let listener = TcpListener::bind(HOST).unwrap();
    println!("Listening started, ready to accept");

    // listen for incoming connections
    for stream in listener.incoming() {
        thread::spawn(move || {
            // create a large set of data [0, 1, .., 255, 0, 1, ..]
            let data: Vec<u8> = (0..TOTAL_BYTES).map(|n| (n % 255) as u8).collect();

            // write entire set of data
            stream.unwrap().write(&data).unwrap();

            println!("Data sent, closing connection");
        });
    }
}
