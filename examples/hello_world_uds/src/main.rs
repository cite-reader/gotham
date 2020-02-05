//! A Hello World example for using Gotham with an `AF_UNIX` listening socket.

use std::convert::Infallible;
use std::io;

use gotham::service::GothamService;
use gotham::state::State;

use hyper::server::accept;
use hyper::service::make_service_fn;
use hyper::Server;

use tokio::net::UnixListener;

const HELLO_WORLD: &str = "Hello World!";

/// The same `Handler` function from the hello_world example.
fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sockets linked to the filesystem linger after the process that created them has vanished,
    // which will block UnixListener::bind. Clean up after any previous run of this program.
    if let Err(e) = std::fs::remove_file("server.ipc") {
        if let io::ErrorKind::NotFound = e.kind() {
            // The file doesn't exist, which we wanted in the first place.
        } else {
            // Some actual problem occured, bail out.
            Err(e)?;
        }
    }
    let mut srv_socket = UnixListener::bind("server.ipc")?;
	println!("Listening for requests on the Unix socket at ./server.ipc");
    let server =
        Server::builder(accept::from_stream(srv_socket.incoming())).serve(make_service_fn(
            |_conn| async move { Ok::<_, Infallible>(GothamService::new(|| Ok(say_hello))) },
        ));

    server.await?;

    Ok(())
}
