use std::{
    path::{Path,PathBuf},
};

// ce fichier va servir a regrouper les reponses 
// il me semble que c le prof nous a demande de faire ca

pub const OPENING_DATA: &str = "150 Opening data connection.\r\n";

// les commandes faites avec succés
pub const WELCOME: &str = "220 Welcome to Rust FTP Server\r\n";
pub const LOGIN_SUCCESS: &str = "230 User logged in.\r\n";
pub const TRANSFER_COMPLETE: &str = "226 Transfer complete.\r\n";
pub const DIR_CHANGED: &str = "250 Directory successfully changed.\r\n";
pub const PASV_MODE: &str = "227 Entering Passive Mode\r\n";
pub const RFNR: &str = "350 File exists and ready to be modified\r\n";
pub const FILE_RENAMED: &str = "250 File renamed with success\r\n";
pub const FILE_DELETED: &str = "250 File deleted successfully\r\n";

// passer en mode passif
pub const USE_PASV_FIRST: &'static str = "425 Use PASV first.\r\n";
pub const DATA_CONNECTION_FAILED: &str = "425 Can’t Open Data Connection.\r\n";

// echec des commandes
pub const FILE_NOT_FOUND: &str = "550 File not found.\r\n";
pub const NOT_A_DIR: &str = "550 Not a directory.\r\n";
pub const DIR_NOT_FOUND: &str = "550 Directory does not exist.\r\n";
pub const FAILED_PASV: &str = "500 Failed to enter passive mode";
pub const UNKNOWN_CMD: &str = "500 Unknown command.\r\n";
pub const RNTO_ERROR: &str = "550 File renamed failed\r\n";
pub const DIR_ALREADY_EXISTS: &str = "500 Directory already exists.\r\n"; 
// reponse de pwd
pub fn current_dir(path: &PathBuf) -> String {

    let path_str = path.to_string_lossy(); 

    let clean_path = if path_str.starts_with("/") {
        path_str.to_string()
    } else {
        format!("/{}", path_str) 
    }; 

    format!("257 \"{}\" is the current directory.\r\n", clean_path)
}

// reponse apres user; demande mdp
pub fn user_ok(username: &str) -> String {
    format!("331 User {} OK. Password required.\r\n", username)
}

// msg d'erreur si on arrive pas aentre en mode passif
pub fn failed_pasv(msg: &str) -> String {
    format!("500 Failed to enter passive mode: {}\r\n", msg)
}

// en cas d'erreur 
pub fn internal_error(msg: &str) -> String {
    format!("500 Internal error: {}\r\n", msg)
}

// entrer dasn le mode mode passif
pub fn pasv_mode(p1: u16, p2: u16) -> String {
    format!("227 Entering Passif Mode (127,0,0,1,{},{}).\r\n", p1, p2)
}

pub fn mkd_ok(dirname: &str) -> String {
    format!("257 {} created.\r\n", dirname)
}

pub fn mkd_del(dirname: &str) -> String {
    format!("257 {} deleted.\r\n", dirname)
}
