use std::{env, process};

#[tokio::main]
async fn main() -> spinta::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Please pass args: <url>");
        process::exit(1);
    }

    let url = &args[1];

    let receiver = spinta::connect(url)?;

    loop {
        while let Some(event) = receiver.try_recv() {
            println!("Received {:?}", event);
        }
    }
}
