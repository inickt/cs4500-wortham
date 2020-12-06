use crate::server::remote_client::RemoteClient;
use crate::server::client::Client;

use std::net::TcpListener;
use std::time::{ Duration, Instant };

const SIGNUP_NAME_TIMEOUT: Duration = Duration::from_secs(10);

const MIN_SIGNUP_PLAYERS: usize = 5;
const MAX_SIGNUP_PLAYERS: usize = 10;

/// Listen for remote player connections on localhost on the given port for a given sign up duration.
///
/// The given client_timeout is how long clients have to respond during a game before they are kicked.
/// The given signup_timeout determines how long each round of waiting for players to join will last.
/// Will sign up a minimum of MIN_SIGNUP_PLAYERS and a max of MAX_SIGNUP_PLAYERS. If the minimum is
/// not reached within the first waiting period, one more waiting period will be run. Once
/// MAX_SIGNUP_PLAYERS have signed up, the waiting period ends.
/// A player will not be signed up if they don't provide their name within SIGNUP_NAME_TIMEOUT.
pub fn signup_clients(port: usize, client_timeout: Duration, signup_timeout: Duration) -> Option<Vec<Box<dyn Client>>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut clients = vec![];
    await_clients(&listener, &mut clients, client_timeout, signup_timeout, SIGNUP_NAME_TIMEOUT);

    if clients.len() < MIN_SIGNUP_PLAYERS {
        await_clients(&listener, &mut clients, client_timeout, signup_timeout, SIGNUP_NAME_TIMEOUT);
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
    signup_timeout: Duration,
    name_timeout: Duration,
) {
    let now = Instant::now();

    // Accept clients and their names in order, blocking for each client until they are
    // both connected and have sent their name. Only then will we try to accept a connection
    // from the next client.
    while now.elapsed() < signup_timeout && clients.len() < MAX_SIGNUP_PLAYERS {
        if let Ok((stream, _)) = listener.accept() {
            let mut remote_client = RemoteClient::new(stream, client_timeout);
            // as long as clients have a valid name we don't care if they are unique
            if remote_client.get_name(name_timeout).is_some() {
                clients.push(Box::new(remote_client));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ai_client::AIClient;
    use crate::client::client_to_server_proxy::ClientToServerProxy;

    const TIMEOUT_200MS: Duration = Duration::from_millis(200);
    const TIMEOUT_1S: Duration = Duration::from_secs(1);

    #[test]
    fn test_no_clients_signup() {
        assert!(signup_clients(8083, TIMEOUT_200MS, TIMEOUT_200MS).is_none());
    }

    #[test]
    fn test_too_few_signup() {
        let threads: Vec<_> = (0..4).map(|_| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                let ai = AIClient::with_zigzag_minmax_strategy();
                let mut client = ClientToServerProxy::new("name".to_string(), Box::new(ai), "127.0.0.1:8084", TIMEOUT_1S)
                    .expect("Unable to create client to server proxy");
                client.send_name().expect("Unable to send name");
            })
        }).collect();

        assert!(signup_clients(8084, TIMEOUT_1S, TIMEOUT_1S).is_none());

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_1_round_signup() {
        let threads: Vec<_> = (0..5).map(|_| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                let ai = AIClient::with_zigzag_minmax_strategy();
                let mut client = ClientToServerProxy::new("name".to_string(), Box::new(ai), "127.0.0.1:8085", TIMEOUT_1S)
                    .expect("Unable to create client to server proxy");
                client.send_name().expect("Unable to send name");
            })
        }).collect();

        assert_eq!(signup_clients(8085, TIMEOUT_1S, TIMEOUT_1S).unwrap().len(), 5);

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_2_round_signup() {
        let threads: Vec<_> = (0..5).map(|num| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                if num > 2 {
                    std::thread::sleep(TIMEOUT_1S);
                }
                let ai = AIClient::with_zigzag_minmax_strategy();
                let mut client = ClientToServerProxy::new("name".to_string(), Box::new(ai), "127.0.0.1:8086", TIMEOUT_1S)
                    .expect("Unable to create client to server proxy");
                client.send_name().expect("Unable to send name");
            })
        }).collect();

        assert_eq!(signup_clients(8086, TIMEOUT_1S, TIMEOUT_1S).unwrap().len(), 5);

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_max_signup() {
        let threads: Vec<_> = (0..12).map(|_| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                let ai = AIClient::with_zigzag_minmax_strategy();
                if let Some(mut client) = ClientToServerProxy::new("name".to_string(), Box::new(ai), "127.0.0.1:8087", TIMEOUT_1S) {
                    client.send_name();
                }
            })
        }).collect();

        assert_eq!(signup_clients(8087, TIMEOUT_1S, TIMEOUT_1S).unwrap().len(), 10);

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_bad_name_signup() {
        let threads: Vec<_> = (0..8).map(|num| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                let ai = AIClient::with_zigzag_minmax_strategy();
                let name = if num == 0 { "" } else { "name" };
                let mut client = ClientToServerProxy::new(name.to_string(), Box::new(ai), "127.0.0.1:8088", TIMEOUT_1S)
                    .expect("Unable to create client to server proxy");
                client.send_name().expect("Unable to send name");
            })
        }).collect();

        assert_eq!(signup_clients(8088, TIMEOUT_1S, TIMEOUT_1S).unwrap().len(), 7);

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_await_clients_name_timeout() {
        let threads: Vec<_> = (0..8).map(|num| {
            std::thread::spawn(move || {
                std::thread::sleep(TIMEOUT_200MS);
                let ai = AIClient::with_zigzag_minmax_strategy();
                let mut client = ClientToServerProxy::new("name".to_string(), Box::new(ai), "127.0.0.1:8089", TIMEOUT_1S)
                    .expect("Unable to create client to server proxy");
                if num != 0 {
                    client.send_name().expect("Unable to send name");
                }
            })
        }).collect();

        let listener = TcpListener::bind("127.0.0.1:8089").unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut clients = vec![];
        await_clients(&listener, &mut clients, TIMEOUT_1S, TIMEOUT_1S, TIMEOUT_200MS);

        assert_eq!(clients.len(), 7);

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
