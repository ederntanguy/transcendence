use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{fs, io};

use clap::{Parser, ValueEnum};
use fern::FormatCallback;
use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use file_rotate::{ContentLimit, FileRotate};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use time::format_description::well_known::Iso8601;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinSet;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use crate::accept_tasks::OnAcceptGenerator;
use crate::match_making::MatchMaker;

mod accept_tasks;
mod game;
mod match_making;
mod protocol;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// Path to the TLS private key for this server's identity.
    tls_private_key: String,

    /// Path to the TLS certificate for this server's identity.
    tls_certificate: String,

    /// Set the port number to bind the listening socket on.
    #[arg(long, short, default_value = "8000")]
    port: u16,

    /// Path of the PostgreSQL socket used to connect to the database engine.
    #[arg(
        long,
        short,
        default_value = "/var/run/postgresql",
        value_name = "PATH"
    )]
    socket_path: String,

    /// Set the folder path.
    ///
    /// The given path can be absolute or relative.
    /// The server will attempt to create all the folders nested in the path.
    #[arg(long, short, default_value = "./log/", value_name = "PATH")]
    log_folder: String,

    /// Set where the printed logging is outputted.
    #[arg(value_enum, long, short, default_value_t)]
    console_channel: ConsoleChannel,
}

#[derive(Copy, Clone, ValueEnum, Default)]
enum ConsoleChannel {
    /// Print to stdout
    #[default]
    Out,
    /// Print to stderr
    Err,
}

/// The tokio-ran main function runs a server listening on port [`PORT`]. All errors are logged, the [`Result`]
/// returned is only given for command-line environments.
#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    setup_logger(cli.log_folder, cli.console_channel)
        .map_err(|e| eprintln!("Error while configuring logging : {e:?}"))?;
    let tls_acceptor = make_tls_acceptor(&cli.tls_private_key, &cli.tls_certificate)
        .map_err(|e| log::error!("Error while creating a TLS config for the server : {e}."))?;
    let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), cli.port);
    log::info!("Server started. Listening on {listen_address}.");
    let db_client = Arc::new(connect_to_db(&cli.socket_path).await?);
    match TcpListener::bind(&listen_address).await {
        Ok(tcp_listener) => {
            let global_match_maker = Arc::new(MatchMaker::new());
            let task_generator = OnAcceptGenerator::new(tcp_listener);
            run_until_signaled(global_match_maker, tls_acceptor, task_generator, db_client).await
        }
        Err(e) => {
            log::error!("Failed to bind to address {listen_address} with error : {e}.");
            Err(())
        }
    }
}

fn make_tls_acceptor(
    tls_private_key_path: &str,
    tls_certificate_path: &str,
) -> Result<TlsAcceptor, Box<dyn Error>> {
    let tls_private_key = PrivatePkcs8KeyDer::from(fs::read(tls_private_key_path)?).into();
    let tls_certificate = CertificateDer::from(fs::read(tls_certificate_path)?);
    let server_config = Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![tls_certificate], tls_private_key)?,
    );
    Ok(TlsAcceptor::from(server_config))
}

/// Set up the global logger to log to stdout/stderr and to a file named as the current timestamp.
fn setup_logger(log_folder: String, console_channel: ConsoleChannel) -> io::Result<()> {
    // Configure log output on the given console
    let console_config = fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .level_for("rustls", log::LevelFilter::Warn)
        .level_for("tungstenite", log::LevelFilter::Warn)
        .level_for("tokio_postgres", log::LevelFilter::Warn)
        .level_for("tokio_tungstenite", log::LevelFilter::Warn)
        .format(format_log);
    let console_config = match console_channel {
        ConsoleChannel::Out => console_config.chain(io::stdout()),
        ConsoleChannel::Err => console_config.chain(io::stderr()),
    };

    // Configure log output in rotating log files
    let rotator = make_rotator(log_folder)?;
    let file_config = fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .level_for("rustls", log::LevelFilter::Debug)
        .level_for("tungstenite", log::LevelFilter::Debug)
        .level_for("tokio_postgres", log::LevelFilter::Debug)
        .level_for("tokio_tungstenite", log::LevelFilter::Debug)
        .format(format_log)
        .chain(rotator as Box<(dyn io::Write + Send)>);

    // Finish the config. Can unwrap because we know we only set the logger once.
    fern::Dispatch::new()
        .chain(console_config)
        .chain(file_config)
        .apply()
        .unwrap();
    Ok(())
}

/// Make the rotating file middleware to give to the logger.
fn make_rotator(log_folder: String) -> io::Result<Box<FileRotate<AppendCount>>> {
    fs::create_dir_all(&log_folder)?;
    let log_file_path = log_folder + "/" + &utc_now_wrapper() + ".log";
    let rotator = Box::new(FileRotate::new(
        log_file_path,
        AppendCount::new(10),
        ContentLimit::Lines(4000),
        Compression::None,
        #[cfg(unix)]
        None,
    ));
    Ok(rotator)
}

/// The function given to the logging crate [`fern`] to format messages.
fn format_log(out: FormatCallback, message: &std::fmt::Arguments, record: &log::Record) {
    out.finish(format_args!(
        "[{} {} {}] {}",
        utc_now_wrapper(),
        record.level(),
        &record
            .target()
            .chars()
            .take_while(|&c| c != ':')
            .collect::<String>(),
        message
    ))
}

/// Create a [`String`] of the current time in the UTC timezone, with a default in case of error.
fn utc_now_wrapper() -> String {
    time::OffsetDateTime::now_utc()
        .format(&Iso8601::DATE_TIME)
        .unwrap_or(String::from("invalid date"))
}

/// Try to establish a connection to a postgresql database through a non-tls and non-password protected socket at the
/// given path.
async fn connect_to_db(socket_path: &str) -> Result<tokio_postgres::Client, ()> {
    let socket_path = match fs::canonicalize(socket_path) {
        Ok(socket_path) => socket_path,
        Err(e) => {
            log::error!(
                "Could not turn the socket path to its canonical absolute form with error : {e}."
            );
            return Err(());
        }
    };
    match tokio_postgres::connect(
        &format!(
            "user=transcendence sslmode=disable host={} port=5432",
            socket_path.display()
        ),
        tokio_postgres::NoTls,
    )
    .await
    {
        Ok((client, connection)) => {
            tokio::spawn(async {
                if let Err(e) = connection.await {
                    log::error!("Database connection worker error : {e}.");
                }
            });
            Ok(client)
        }
        Err(e) => {
            log::error!("Failed to connect to the database with error : {e}.");
            Err(())
        }
    }
}

/// Create asynchronous tasks to handle connections until an interrupt or terminate signal is received.
/// The return value of the tasks spawned are ignored.
async fn run_until_signaled(
    match_maker: Arc<MatchMaker<TlsStream<TcpStream>>>,
    tls_acceptor: TlsAcceptor,
    mut task_generator: OnAcceptGenerator,
    db_client: Arc<tokio_postgres::Client>,
) -> Result<(), ()> {
    let (mut sigint_handler, mut sigterm_handler) = match signal(SignalKind::interrupt())
        .and_then(|si| signal(SignalKind::terminate()).map(|st| (si, st)))
    {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create the signal handlers with error : {e:?}.");
            return Err(());
        }
    };
    let mut task_set = JoinSet::new();
    let res = loop {
        tokio::select! {
            biased;
            signal = sigint_handler.recv() => match signal {
                Some(()) => {
                    log::info!("Received an interrupt signal.");
                    break Ok(());
                }
                None => {
                    log::error!("The interrupt signal handler stopped working, have to stop now.");
                    break Err(());
                }
            },
            signal = sigterm_handler.recv() => match signal {
                Some(()) => {
                    log::info!("Received a terminate signal.");
                    break Ok(());
                }
                None => {
                    log::error!("The terminate signal handler stopped working, have to stop now.");
                    break Err(());
                }
            },
            task_generation_result = task_generator.generate_next_task(
                &tls_acceptor,
                &mut task_set,
                |websocket, id| protocol::execute_protocol_on_connection(
                    websocket,
                    id,
                    match_maker.clone(),
                    db_client.clone()
                )
            ) => {
                if task_generation_result.is_err() {
                    break Err(());
                }
            },
        }
    };
    log::info!("Closing all connections and shutting down spawned tasks...");
    task_set.shutdown().await;
    log::info!("Done, exiting.");
    res
}
