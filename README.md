# What is this?

This is a demonstration of the performance characteristics of `Read` vs `BufRead` in Rust. It was created to go with [this blog post](http://localhost:3000/blog/bufread-and-when-to-use-it).

# Running the demo

This project contains both a TCP server and a TCP client. You can start each of them on your machine, and when the client is started it will request a stream of data from the server and then report how long it took with and without `BufReader`.
With the code cloned, open two terminals and navigate both to the project directory. In the first one, type:


```
cargo run server
```

It should give you something like this:

```
  Compiling read-demo v0.1.0 (/Users/my-profile/read-demo)
    Finished dev [unoptimized + debuginfo] target(s) in 0.79s
     Running `target/debug/read-demo server`
Listening started, ready to accept
```

Now in the other terminal, type:

```
cargo run client
```

The program will start in client-mode, find the local server in the other terminal, connect to it, and request a stream of bytes. It will then do the same thing with a `BufReader`. It will record the time each of these takes, and print it to the console. You should get something like:

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/read-demo client`
TcpStream (Read) took 469ms
BufReader<TcpStream> (BufRead) took 109ms
```
Of course your numbers won't match mine exactly, but the `BufReader` one should be significantly faster.

# How it works

Here's what happens when we start up the server:

```rust
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
```

We open a TCP socket on localhost, and wait for incoming connections. When one is made, we spawn a new thread, generate a 1MB array of junk bytes (0-255 over and over), and send it all over the socket.

The client is where the relevant part happens. We run the client logic twice, once without and then once with a `BufReader`:


```rust
fn client() {
    // test with a raw TCP stream
    client_inner("TcpStream (Read)", TcpStream::connect(HOST).unwrap());
    // test with the stream wrapped in a buffered reader
    client_inner(
        "BufReader<TcpStream> (BufRead)",
        BufReader::new(TcpStream::connect(HOST).unwrap()),
    );
}
```

The core logic is then the same between the two:

```rust
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
```

We create a buffer large enough to hold the expected payload. Then we loop, reading exactly one byte at a time from the stream on each iteration. Once we've stopped receiving bytes (the stream has been closed), we break out of the loop and print how long the whole thing took.
