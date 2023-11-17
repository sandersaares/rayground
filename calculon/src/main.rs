use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

// We are writing a calculation system. You connect via TCP and send commands to modify some global state.
// There is a global variable X and there are commands to modify it.
// The commands are:
// ADD 123
// SUBTRACT 123
// POWER 2.5 - raise X to power
// SHOW - displays value of X

#[derive(Debug, Default)]
struct GlobalState {
    x: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let global_state = Arc::new(Mutex::new(GlobalState::default()));
    let listener = TcpListener::bind("127.0.0.1:4673").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let global_state = global_state.clone();

        tokio::spawn(async move {
            if let Err(e) = process_request(stream, global_state).await {
                eprintln!("Failed to process request; error = {}", e);
            }
        });
    }
}

async fn process_request(
    stream: TcpStream,
    global_state: Arc<Mutex<GlobalState>>,
) -> Result<(), Box<dyn Error>> {
    let (read_stream, mut write_stream) = split(stream);

    let reader = BufReader::new(read_stream);
    let mut lines = reader.lines();

    write_stream
        .write_all("ADD 1.23/SUBTRACT 1.23/POWER 1.23/SHOW\r\n".as_bytes())
        .await?;

    while let Some(line) = lines.next_line().await? {
        println!("Received line: {}", line);

        let words: Vec<_> = line.split_whitespace().collect();

        if words.is_empty() {
            continue;
        }

        match words[0] {
            "ADD" => {
                if words.len() != 2 {
                    eprintln!("ADD command requires exactly one argument.");
                    continue;
                }

                let operand = words[1].parse::<f64>()?;
                let new_value = add(operand, &global_state);
                write_stream
                    .write_all(format!("X += {operand} = {new_value}\r\n").as_bytes())
                    .await?;
            }
            "SUBTRACT" => {
                if words.len() != 2 {
                    eprintln!("SUBTRACT command requires exactly one argument.");
                    continue;
                }

                let operand = words[1].parse::<f64>()?;
                let new_value = subtract(operand, &global_state);
                write_stream
                    .write_all(format!("X -= {operand} = {new_value}\r\n").as_bytes())
                    .await?;
            }
            "POWER" => {
                if words.len() != 2 {
                    eprintln!("POWER command requires exactly one argument.");
                    continue;
                }

                let operand = words[1].parse::<f64>()?;
                let new_value = power(operand, &global_state);
                write_stream
                    .write_all(format!("X ^= {operand} = {new_value}\r\n").as_bytes())
                    .await?;
            }
            "SHOW" => {
                if words.len() != 1 {
                    eprintln!("SHOW command requires exactly zero arguments.");
                    continue;
                }

                let value = show(&global_state);
                write_stream
                    .write_all(format!("X = {value}\r\n").as_bytes())
                    .await?;
            }
            _ => {
                write_stream
                    .write_all(format!("Unknown command: {}\r\n", words[0]).as_bytes())
                    .await?;
            }
        }
    }

    Ok(())
}

fn add(value: f64, global_state: &Arc<Mutex<GlobalState>>) -> f64 {
    let mut guarded_state = global_state.as_ref().lock().unwrap();
    let new_value = guarded_state.x + value;
    guarded_state.x = new_value;

    new_value
}

fn subtract(value: f64, global_state: &Arc<Mutex<GlobalState>>) -> f64 {
    let mut guarded_state = global_state.as_ref().lock().unwrap();
    let new_value = guarded_state.x - value;
    guarded_state.x = new_value;

    new_value
}

fn power(value: f64, global_state: &Arc<Mutex<GlobalState>>) -> f64 {
    let mut guarded_state = global_state.as_ref().lock().unwrap();
    let new_value = guarded_state.x.powf(value);
    guarded_state.x = new_value;

    new_value
}

fn show(global_state: &Arc<Mutex<GlobalState>>) -> f64 {
    let guarded_state = global_state.as_ref().lock().unwrap();
    guarded_state.x
}
