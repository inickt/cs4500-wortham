use crate::server::serverclient::ClientProxy;
use crate::server::connection::PlayerConnection;

use std::net::TcpListener;
use std::time::{ Duration, Instant };

const SIGNUP_TIMEOUT: Duration = Duration::from_secs(30);

const MIN_SIGNUP_PLAYERS: usize = 5;
const MAX_SIGNUP_PLAYERS: usize = 10;

/*
enum ServerToClientMessage {
    Start(bool),
    PlayingAs(JSONPenguinColor),
    PlayingWith(Vec<JSONPenguinColor>),
    Setup(JSONGameState),
    TakeTurn(JSONGameState, Vec<Action>),
    End(bool),
}
*/


/* referee flow
for player in players {
    player.send(play as message)
}

for player in players {
    player.send(play with message)
}

setup...
<- place

take-turn...
<- action
*/

pub fn signup_clients(port: &str) -> Option<Vec<ClientProxy>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut clients = vec![];
    await_clients(&listener, &mut clients);

    if clients.len() < MIN_SIGNUP_PLAYERS {
        await_clients(&listener, &mut clients);
    }

    // If we still don't have enough players then give up and return None
    if clients.len() < MIN_SIGNUP_PLAYERS {
        None
    } else {
        Some(clients)
    }
}


fn await_clients(listener: &TcpListener, clients: &mut Vec<ClientProxy>) {
    let now = Instant::now();

    // Accept clients and their names in order, blocking for each client until they are
    // both connected and have sent their name. Only then will we try to accept a connection
    // from the next client.
    while now.elapsed() < SIGNUP_TIMEOUT && clients.len() < MAX_SIGNUP_PLAYERS {
        if let Ok((stream, _)) = listener.accept() {
            let mut connection = PlayerConnection::new(stream);
            if connection.receive_name().is_some() {
                let proxy = ClientProxy::Remote(connection);
                clients.push(proxy);
            }
        }
    }
}
