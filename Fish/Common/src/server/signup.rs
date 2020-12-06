use crate::server::remote_client::RemoteClient;
use crate::server::client::Client;

use std::net::TcpListener;
use std::time::{ Duration, Instant };

const SIGNUP_NAME_TIMEOUT: Duration = Duration::from_secs(10);

const MIN_SIGNUP_PLAYERS: usize = 5;
const MAX_SIGNUP_PLAYERS: usize = 10;

pub fn signup_clients(port: usize, client_timeout: Duration, signup_timeout: Duration) -> Option<Vec<Box<dyn Client>>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut clients = vec![];
    await_clients(&listener, &mut clients, client_timeout, signup_timeout);

    if clients.len() < MIN_SIGNUP_PLAYERS {
        await_clients(&listener, &mut clients, client_timeout, signup_timeout);
    }

    // If we still don't have enough players then give up and return None
    if clients.len() < MIN_SIGNUP_PLAYERS {
        None
    } else {
        Some(clients)
    }
}

fn await_clients(
    listener: &TcpListener, 
    clients: &mut Vec<Box<dyn Client>>, 
    client_timeout: Duration, 
    signup_timeout: Duration
) {
    let now = Instant::now();

    // Accept clients and their names in order, blocking for each client until they are
    // both connected and have sent their name. Only then will we try to accept a connection
    // from the next client.
    while now.elapsed() < signup_timeout && clients.len() < MAX_SIGNUP_PLAYERS {
        if let Ok((stream, _)) = listener.accept() {
            let mut remote_client = RemoteClient::new(stream, client_timeout);
            // as long as clients have a valid name we don't care if they are unique
            if remote_client.get_name(SIGNUP_NAME_TIMEOUT).is_some() {
                clients.push(Box::new(remote_client));
            }
        }
    }
}
