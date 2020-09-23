mod xjson;

use std::net::TcpListener;
use std::io::Write;
use std::error::Error;
use std::time::{Instant, Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let port = std::env::args().nth(1).unwrap_or("4567".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;

    listener.set_nonblocking(true)?;
    let start_time = Instant::now();
    let duration_of_loop = Duration::from_secs(3);

    loop {
        if start_time.elapsed() > duration_of_loop {
            break;
        }

        if let Ok((mut stream, _)) = listener.accept() {
            let output = xjson::xjson(&stream);
            let output_with_newline = format!("{}\n", output);
            stream.write(output_with_newline.as_bytes())?;
            break; // don't continue looping if we already connected to a client
        };
    }

    Ok(())
}
