# sta-server

This crate handles the server side actions for server-client app

## Server hangling function:

handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>)

