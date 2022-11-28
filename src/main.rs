use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::quinn_helpers::{make_client_endpoint, make_server_endpoint};

mod quinn_helpers;

async fn init_connection(is_server: bool, expected_num_accepts_uni: u32) -> Result<(), Box<dyn Error>> {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let client_addr = "127.0.0.1:5001".parse().unwrap();

    if is_server {
        let (endpoint, server_cert) = make_server_endpoint(server_addr)?;

        // Single connection
        let incoming_conn = endpoint.accept().await.unwrap();
        let conn = incoming_conn.await.unwrap();
        println!("[server] connection accepted: addr={}", conn.remote_address());

        let mut send = conn
            .open_uni()
            .await?;
        println!("[server] opened uni stream");
        send.write_u32(0).await?;
        send.flush().await?;
        println!("[server] wrote bytes");

        loop {
            // Loop to keep server alive
        }
    } else {
        // Bind this endpoint to a UDP socket on the given client address.
        let mut endpoint = make_client_endpoint(client_addr, &[])?;

        // Connect to the server passing in the server name which is supposed to be in the server certificate.
        let connection = endpoint.connect(server_addr, "localhost")?.await?;
        println!("[client] connected: addr={}", connection.remote_address());

        for _ in 0..expected_num_accepts_uni {
            println!("[client] waiting for uni stream");
            let mut recv = connection.accept_uni().await?;
            println!("[client] accepted uni stream");
            let id = recv.read_u32().await?;
        }

        println!("[client] Initialized!");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let server = tokio::spawn(async {
        init_connection(true, 1).await.unwrap();
    });
    let client = init_connection(false, 1).await;
    println!("{:#?}", client);
    assert!(client.is_ok());
}
