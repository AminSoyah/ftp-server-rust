#![allow(unused)] // delete that to clear code at the end 

use std::{
    vec::{Vec}, 
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc},
};

use log::{debug, error, log_enabled, info, Level};
use clap::Parser;  

pub mod ftp_responses;
pub mod listener;
pub mod errors;
pub mod client;
pub mod user_session;
pub mod commands;



#[derive(Parser)]
#[command(
    version, 
    about, 
    long_about = None, 
    override_usage = "ftp-server [MANDATORY] -d <dir> [OPTIONAL] -u <USR> --pwd <PWD> -p <PORT> -i <IP> [OPTIONS]"
)]
struct Cli {
    /// [MANDATORY] Starting directory for the FTP server 
    #[arg(short, long, required=true)]
    dir: String, 
    /// Username used for connection to FTP server 
    #[arg(short, long, default_value_t =("anonymous".to_string()))] 
    usr: String,
    /// Password used for connection 
    #[arg(long, default_value_t=("guest".to_string()))]
    pwd: String,
    /// Port used for connection to FTP server 
    #[arg(short, long, default_value_t=32638)]
    port: i32, 
    /// IP used for connection to FTP server 
    #[arg(short, long, default_value_t=IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    ip: IpAddr, 
    /// Sets the max number of threads for the server thread-pool
    #[arg(short, long, default_value_t=20)]
    nb_threads: usize, 
} 


fn main() -> std::io::Result<()> {

    let args = Cli::parse();
    
    let username = args.usr; 
    let password = args.pwd; 
    let port = args.port; 
    let ip = args.ip; 
    let nb_threads = args.nb_threads;
    let starting_directory = args.dir;

    env_logger::init();

    let server = Arc::new(listener::ServerListener::new(ip.to_string(), port, nb_threads, starting_directory.to_string())); 
    info!("> server created ...");

    let _ = server.listen_clients().expect("Failed to start server");

    Ok(())
}
