use ftp_server::user_session::UserSession;
use std::path::PathBuf;
use std::net::TcpListener;
use ftp_server::client::FtpClient;
use ftp_server::commands::FtpCommand;
use std::net::TcpStream;

// Test que resolve_path renvoie le répertoire racine de la session
#[test]
fn test_resolve_path_root() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());
    
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    
    std::thread::spawn(move || {
        let _ = std::net::TcpStream::connect(addr);
    });
    
    let (stream, _) = server.accept().unwrap();
    let client = ftp_server::client::FtpClient::new(session, stream);
    
    let result = client.resolve_path();
    assert_eq!(result, PathBuf::from("./ftp_root"));
}

// Test que resolve_any_path renvoie le chemin absolu correct pour un chemin virtuel
#[test]
fn test_resolve_any_path_absolute() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());
    
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    
    std::thread::spawn(move || {
        let _ = std::net::TcpStream::connect(addr);
    });
    
    let (stream, _) = server.accept().unwrap();
    let client = ftp_server::client::FtpClient::new(session, stream);
    
    let virtual_path = PathBuf::from("/photos");
    let result = client.resolve_any_path(&virtual_path);
    
    assert!(result.to_string_lossy().contains("ftp_root"));
    assert!(result.to_string_lossy().contains("photos"));
}

// Test que absolute_or_relative renvoie un chemin relatif correct quand on passe un chemin relatif
#[test]
fn test_absolute_or_relative_absolute() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());
    
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    
    std::thread::spawn(move || {
        let _ = std::net::TcpStream::connect(addr);
    });
    
    let (stream, _) = server.accept().unwrap();
    let client = ftp_server::client::FtpClient::new(session, stream);
    
    let result = client.absolute_or_relative("/test.txt".to_string());
    assert_eq!(result, PathBuf::from("/test.txt"));
}

// Test que absolute_or_relative renvoie un chemin relatif correct quand on passe un chemin relatif
#[test]
fn test_absolute_or_relative_relative() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());
    
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    
    std::thread::spawn(move || {
        let _ = std::net::TcpStream::connect(addr);
    });
    
    let (stream, _) = server.accept().unwrap();
    let client = ftp_server::client::FtpClient::new(session, stream);
    
    let result = client.absolute_or_relative("test.txt".to_string());
    assert!(result.to_string_lossy().contains("test.txt"));
}

// Test que setup_passive_mode ouvre un port et renvoie Ok
#[test]
fn test_setup_passive_mode_opens_port() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());
    
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    
    std::thread::spawn(move || {
        let _ = std::net::TcpStream::connect(addr);
    });
    
    let (stream, _) = server.accept().unwrap();
    let mut client = ftp_server::client::FtpClient::new(session, stream);
    
    let result = client.setup_passive_mode();
    assert!(result.is_ok());
}

// Test que le calcul des ports pour le mode passif est correct
#[test]
fn test_passive_mode_calculates_port() {
    let port: u16 = 50234;
    let p1 = port / 256;
    let p2 = port % 256;
    
    assert_eq!(p1, 196);
    assert_eq!(p2, 58);
    
    let reconstructed_port = (p1 as u16) * 256 + (p2 as u16);
    assert_eq!(reconstructed_port, port);
}

// Test simple de handle_command pour la commande TYPE
#[test]
fn test_handle_command_type() {
    let session = UserSession::init_id_dir(1, "./ftp_root".to_string());

    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();

    std::thread::spawn(move || {
        let _ = TcpStream::connect(addr);
    });

    let (stream, _) = server.accept().unwrap();
    let mut client = FtpClient::new(session, stream);

    let result = client.handle_command(FtpCommand::Type);
    assert!(result.is_ok());

}