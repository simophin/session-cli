use crate::rand_util::RandExt;
use clap::{Parser, Subcommand};

mod api;
mod ffi;
mod hex_encode;
mod mnemonic;
mod rand_util;

extern crate link_cplusplus;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    GenKey,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    env_logger::init();

    // let Cli { commands } = Cli::parse();
    // match commands {
    //     Commands::GenKey => {
    //         let (public_key, secret_key) = ffi::ed25519::session_ed25519_key_pair().expect("Failed to generate key pair");
    //         println!("Public key: {}", hex::encode(public_key));
    //         println!("Secret key: {}", hex::encode(secret_key));
    //     }
    // }

    let (public_key, secret_key) =
        ffi::curve25519::session_curve25519_key_pair().expect("Failed to generate key pair");
    let pub_key = hex::encode(&public_key.0);

    let seed_service_node = api::seed::get_service_nodes(3)
        .await
        .expect("To get service nodes")
        .rand_ref()
        .clone();

    let swarm_nodes = api::service_node::get_swarm_nodes(&seed_service_node, &pub_key)
        .await
        .expect("To get swarm nodes");

    log::info!("Service nodes: {swarm_nodes:#?}");
}
