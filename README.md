# server-client


Client/Server application that allows you to broadcast data from clients to all the other
clients connected to the server

## Example:

cargo run -- --server --host 127.0.0.1 --port 12345

This command runs the app in the server mode listening on localhost port 12345

cargo run -- --client --host 127.0.0.1 --port 12345

This command runs the app in the client mode connecting to localhost on port 12345

If you need to debug the app you can enable logs by changing the 'RUST_LOG' variable

RUST_LOG=info cargo run -- --server --host 127.0.0.1 --port 12345
