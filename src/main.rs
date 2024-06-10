//!
//! Client/Server application that allows you to broadcast data from clients to all the other
//! clients connected to the server

use clap::{command, Parser, ArgGroup};
use std::net::{TcpStream, TcpListener};
use std::sync::{ Arc, Mutex };
use std::thread;
use log::{info, error};
use env_logger;
use std::process;
use std::io::{Read, Write};
use sta_client::{ strip_to_second_space, parse_command, create_dir };
use sta_server::handle_client;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("mode")
    .required(true)
    .args(&["client", "server"]),
))]
struct Args {
    // IP address for client/server connection
    #[arg(long, value_name = "HOST")]
    host: String,
    /// Port for the connection
    #[arg(long, value_name = "PORT")]
    port: i32,
    /// Run as client
    #[arg(long)]
    client: bool,
    /// Run as server
    #[arg(long)]
    server: bool,
}

/// Parsing program arguments
fn parse_arguments() -> Args {
    let args = Args::parse();

    // Validate the PORT value
    if !(0..=65535).contains(&args.port) {
        eprintln!("Error: Port must be between 0 and 65535.");
        std::process::exit(1);
    }

    args
}

/// Main program entry where we decide the behaviour of the app based on parsed arguments
fn main() {

    env_logger::init();

    // Parse taken arguments
    let options = parse_arguments();

    if options.server {
        // Server part of the application

        // Create socket
        let address = format!("{}:{}",options.host, options.port);
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind to address {}: {}", address, e);
                process::exit(1);
            }
        };

        info!("Successfully bound to address: {}", address);

        // Create vector of connections
        let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

        // Iterate through the connection
        for stream in listener.incoming() {
            let stream = stream.expect("Failed to accept connection");

            // Log client connection
            match stream.peer_addr() {
                Ok(addr) => info!("Client connected from address: {}", addr),
                Err(e) => error!("Failed to get client address: {}", e),
            };

            let clients = Arc::clone(&clients);
            clients.lock().unwrap().push(stream.try_clone().expect("Failed to clone stream"));

            // Spawn a thread for the active connection
            thread::spawn(move || {
                if let Err(e) = handle_client(stream, clients) {
                    error!("Error in client handler: {:?}", e);
                }
            });
        }


    } else {

        // Client part of the application

        // Define connection address
        let address = format!("{}:{}",options.host, options.port);

        // Connect to the server
        let mut stream = match TcpStream::connect(&address) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed connect to address {}: {}", address, e);
                process::exit(1);
            }
        };

        info!("Successfully connected to address: {}", address);

        // Use Arc and Mutex to share the stream between threads
        let stream_clone = stream.try_clone().expect("Failed to clone stream!");

        let stream_arc = Arc::new(Mutex::new(stream_clone));

        // Clone the stream for the sender thread
        let sender_stream = Arc::clone(&stream_arc);
        thread::spawn(move || {
            thread::spawn(move || {
                loop {
                    if let Some((command, argument)) = parse_command() {
                        let message = format!("{} {}", command, argument);
                        let size = message.len();

                        let mut stream = match sender_stream.lock() {
                            Ok(stream) => stream,
                            Err(e) => {
                                error!("Failed to lock stream: {}", e);
                                process::exit(1);
                            }
                        };

                        // Send the command and size
                        if let Err(e) = stream.write_all(format!("{} {}\n", command, size).as_bytes()) {
                            error!("Failed to write to stream: {}", e);
                            process::exit(1);
                        }
                        if let Err(e) = stream.write_all(message.as_bytes()) {
                            error!("Failed to write to stream: {}", e);
                            process::exit(1);
                        }
                    } else {
                        println!("No command entered. Exiting...");
                        break;
                    }
                }
            });
        });

        // Clone the stream for the reader thread
        let _reader_stream = Arc::clone(&stream_arc);
        thread::spawn(move || {
            loop {
                loop {

                    // Read the command and size from the server
                    let mut buffer = [0; 1024]; // Buffer size can be adjusted as needed
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                // Connection closed by server
                                println!("Server closed the connection. Exiting...");
                                process::exit(1);
                            }

                            // Parse the received data
                            let data = String::from_utf8_lossy(&buffer[..n]);
                            let parts: Vec<&str> = data.trim().splitn(2, ' ').collect();

                            if parts.len() == 2 {

                                let command = parts[0];
                                let size_str = parts[1];
                                if let Ok(size) = size_str.parse::<usize>() {

                                    // Read the data payload of specified size
                                    let mut payload = vec![0; size];
                                    match stream.read_exact(&mut payload) {
                                        Ok(_) => {
                                            let data = strip_to_second_space(String::from_utf8_lossy(&payload));
                                            match command {
                                                ".text" => {
                                                    println!("{}", data);
                                                }
                                                ".image" => {
                                                    let _parts: Vec<&str> = data.trim().splitn(2, ' ').collect();
                                                    match create_dir("images") {
                                                        Ok(()) => {
                                                            info!("Directory 'images' ready...");
                                                        }
                                                        Err(e) => {
                                                            error!("Could not create a directory : {} ", e);
                                                        }
                                                    }
                                                }
                                                ".file" => {
                                                    let _parts: Vec<&str> = data.trim().splitn(2, ' ').collect();
                                                    let _file = parts[0];
                                                    let _content = parts[1];

                                                    match create_dir("files") {
                                                        Ok(()) => {
                                                            info!("Directory 'files' ready...");
                                                        }
                                                        Err(e) => {
                                                            error!("Could not create a directory : {} ", e);
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    println!("Received command: none");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to read payload: {}", e);
                                            continue;
                                        }
                                    }
                                } else {
                                    error!("Failed to parse size: {}", size_str);
                                    continue;
                                }
                            } else {
                                error!("Invalid data received from server");
                                process::exit(1);
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from stream: {}", e);
                            process::exit(1);
                        }
                    }
                }

            }
        });

        // Keep the main thread alive to keep the client running
        loop {
            // Perform any main thread tasks if necessary
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

    }

}
