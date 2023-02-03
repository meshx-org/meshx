use std::env;
use std::fs;
use std::io::{Read, Write};
use wasm_tcp::public_function;

use tikv_client::{Config, TransactionClient};

fn call_dynamic() -> u32 {
    public_function()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let s = call_dynamic();
    println!("{}", s);

    let config = Config::new();

    config.with_security(
        // The path to the file that contains the PEM encoding of the server’s CA certificates.
        "/path/to/ca.pem",
        // The path to the file that contains the PEM encoding of the server’s certificate chain.
        "/path/to/client-cert.pem",
        // The path to the file that contains the PEM encoding of the server’s private key.
        "/path/to/client-key.pem",
    );

    let txn_client = TransactionClient::new(vec!["127.0.0.1:2379"]).await?;

    let mut txn = txn_client.begin_optimistic().await?;
    txn.put("key".to_owned(), "hello".to_owned()).await?;
    let value = txn.get("key".to_owned()).await?;
    txn.commit().await?;

    println!("{:?}", value);

    Ok(())
}
