use ftp_server::commands::FtpCommand;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Test que la commande USER est correctement parsée
#[test]
fn test_parse_user() {
    let result = FtpCommand::parse_ftp_command("USER alice");
    assert!(result.is_some());
}

// Test que la commande LIST est correctement parsée
#[test]
fn test_parse_list() {
    let result = FtpCommand::parse_ftp_command("LIST");
    assert!(result.is_some());
}

// Test que la commande PWD est correctement parsée
#[test]
fn test_parse_pwd() {
    let result = FtpCommand::parse_ftp_command("PWD");
    assert!(result.is_some());
}

// Test que ftp_command_pwd renvoie une réponse contenant le code 257
#[test]
fn test_pwd() {
    let path = PathBuf::from("/test");
    let result = FtpCommand::ftp_command_pwd(&path);
    assert!(result.contains("257"));
}

// Test que ftp_command_cwd réussit pour un répertoire existant
#[test]
fn test_cwd_success() {
    let temp_dir = TempDir::new().unwrap();
    let result = FtpCommand::ftp_command_cwd(&temp_dir.path().to_path_buf());
    assert!(result.is_ok());
}

// Test que ftp_command_cwd échoue pour un répertoire inexistant
#[test]
fn test_cwd_not_exists() {
    let path = PathBuf::from("/inexistant999");
    let result = FtpCommand::ftp_command_cwd(&path);
    assert!(result.is_err());
}

// Test que ftp_command_list fonctionne pour un répertoire vide
#[test]
fn test_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let result = FtpCommand::ftp_command_list(&temp_dir.path().to_path_buf());
    assert!(result.is_ok());
}

// Test que ftp_command_list liste correctement les fichiers présents dans le répertoire
#[test]
fn test_list_with_files() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "hello").unwrap();
    let result = FtpCommand::ftp_command_list(&temp_dir.path().to_path_buf());
    assert!(result.unwrap().contains("test.txt"));
}