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

        let maybe_stream = listener.accept();
        if let Ok((mut stream, _)) = maybe_stream {
            let output = xjson::xjson(&stream);
            println!("post output");
            stream.write(output.as_bytes())?;
            println!("post streamwrite");
        };
    }

    Ok(())
}
