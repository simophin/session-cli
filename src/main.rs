// extern crate link_cplusplus;

extern crate link_cplusplus;

use crate::clock::ClockSource;
use crate::config_state::ConfigState;
use crate::db::app_setting::AppSettingRepositoryExt;
use crate::identity::Identity;
use crate::network::batch::BatchManager;
use crate::network::legacy::LegacyNetwork;
use crate::network::swarm::{SwarmAuth, SwarmManager, SwarmState};
use crate::oxenss::namespace::{
    ContactsNamespace, ConvoInfoVolatileConfigNamespace, DefaultNamespace,
    UserGroupsConfigNamespace, UserProfileConfigNamespace,
};
use crate::worker::{sync_config, sync_groups, sync_messages};
use clap::{Parser, Subcommand};
use r2d2_sqlite::SqliteConnectionManager;
use reqwest::Client;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::try_join;

mod oxenss;

mod base64;
#[allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    non_upper_case_globals,
    improper_ctypes
)]
mod bindings;
mod config;
mod curve25519;
mod ed25519;
mod hex_encode;
mod identity;
mod ip;

#[macro_use]
mod key;
mod clock;
mod config_state;
mod crypto;
mod cwrapper;
mod db;
mod io;
mod logging;
mod message_crypto;
mod mnemonic;
// mod network;
mod app_setting;
mod blinding;
mod http_api;
mod network;
mod service;
mod session_id;
mod sogs_api;
mod utils;
mod worker;

mod protos {
    include!(concat!(env!("OUT_DIR"), "/session_protos.rs"));
    include!(concat!(env!("OUT_DIR"), "/session_protos.serde.rs"));
    include!(concat!(env!("OUT_DIR"), "/web_socket_protos.rs"));
    include!(concat!(env!("OUT_DIR"), "/web_socket_protos.serde.rs"));
    include!(concat!(env!("OUT_DIR"), "/session_cli_app.rs"));
    include!(concat!(env!("OUT_DIR"), "/session_cli_app.serde.rs"));
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    GenKey,
    RetrieveConfigMessages {
        #[clap(short, long, env)]
        mnemonic: String,
    },
}

const CONFIG_POLL_INTERVAL: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    env_logger::init();

    // unsafe {
    //     bindings::session_add_logger_full(Some(logging::log_session));
    //     if let Some(level) = log::max_level().to_level() {
    //         bindings::session_logger_set_level_default(logging::log_level_to_session_log_level(
    //             level,
    //         ));
    //     }
    // }

    let db_file = dirs::home_dir().unwrap().join("Temp/session.sqlite3db");
    let _ = std::fs::remove_file(&db_file);

    let repo = db::Repository::new(SqliteConnectionManager::file(db_file))
        .expect("To create a new repository");

    let network = LegacyNetwork::new(
        Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap(),
        non_empty_vec![
            "https://seed1.getsession.org/".parse().unwrap(),
            "https://seed2.getsession.org/".parse().unwrap()
        ],
    );

    let identity = Identity::from_mnemonic(&std::env::var("MNEMONIC").expect("To have a mnemonic"))
        .expect("To create an identity");

    repo.obtain_connection()
        .unwrap()
        .save_setting(None, &identity)
        .expect("To save identity");

    let swarm_state = SwarmState::new(
        identity
            .session_id()
            .into_owned()
            .try_into()
            .expect("To convert"),
    );
    let config_state =
        ConfigState::new(&repo, identity.ed25519_sec_key()).expect("To create a new config state");

    let (manual_poll_trigger_tx, manual_poll_trigger_rx) = broadcast::channel(1);
    let clock_source = ClockSource::default();

    let gen_blinded_ids = worker::gen_blinded_ids::gen_blinded_ids(
        &identity,
        config_state.user_groups_config.subscribe(),
        &repo,
    );

    // let calibrate_clock = async {
    //     let mut rx = network.subscribe_clock_calibration();
    //     loop {
    //         if let Some((instant, timestamp)) = *rx.borrow() {
    //             clock_source.submit_calibration(instant, timestamp);
    //         }
    //
    //         let _ = rx.changed().await?;
    //     }
    //
    //     anyhow::Ok(())
    // };

    let swarm_manager = SwarmManager::new(&network);
    let (batch_manager, runner) = BatchManager::new(&swarm_manager);

    let run_batch = batch_manager.run(runner);

    let poll = sync_messages::<DefaultNamespace, _>(
        &repo,
        &batch_manager,
        swarm_state.clone(),
        &identity,
        CONFIG_POLL_INTERVAL,
        manual_poll_trigger_rx,
        &clock_source,
    );

    let print_logs = config_state.log_configs();

    let sync_user_profile = sync_config::<UserProfileConfigNamespace, _>(
        &batch_manager,
        swarm_state.clone(),
        None,
        &config_state.user_profile_config,
        CONFIG_POLL_INTERVAL,
        manual_poll_trigger_tx.subscribe(),
        &repo,
        &identity,
        &clock_source,
    );

    let sync_user_groups = sync_config::<UserGroupsConfigNamespace, _>(
        &batch_manager,
        swarm_state.clone(),
        None,
        &config_state.user_groups_config,
        CONFIG_POLL_INTERVAL,
        manual_poll_trigger_tx.subscribe(),
        &repo,
        &identity,
        &clock_source,
    );

    let sync_convo_info_config = sync_config::<ConvoInfoVolatileConfigNamespace, _>(
        &batch_manager,
        swarm_state.clone(),
        None,
        &config_state.convo_info_volatile_config,
        CONFIG_POLL_INTERVAL,
        manual_poll_trigger_tx.subscribe(),
        &repo,
        &identity,
        &clock_source,
    );

    let sync_contacts = sync_config::<ContactsNamespace, _>(
        &batch_manager,
        swarm_state.clone(),
        None,
        &config_state.contacts_config,
        CONFIG_POLL_INTERVAL,
        manual_poll_trigger_tx.subscribe(),
        &repo,
        &identity,
        &clock_source,
    );

    let sync_groups = sync_groups(
        &batch_manager,
        &identity,
        &repo,
        config_state.user_groups_config.subscribe(),
        manual_poll_trigger_tx.subscribe(),
        &clock_source,
    );

    try_join!(
        print_logs,
        run_batch,
        sync_user_profile,
        poll,
        sync_user_groups,
        sync_convo_info_config,
        sync_contacts,
        sync_groups,
        gen_blinded_ids,
    )
    .unwrap();
}
