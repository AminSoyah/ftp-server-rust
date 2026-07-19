
use std::{
    net::TcpStream,
    io,
    fs,
    fs::{File ,DirEntry},
    path::{Path, PathBuf},
    io::{Read, Write,BufRead,BufReader, BufWriter},
    net::TcpListener,
    env,
};

use crate::{
    user_session::UserSession,
    ftp_responses as FtpResponse
};
use log::{debug, error, log_enabled, info, Level};
use chrono::{DateTime, Local};
use tar::{Builder, Archive};

#[derive(Debug)]
pub enum FtpCommand {
    Usr(String),
    Pass(String), 
    Cwd(String), 
    Pwd,
    List, 
    Pasv,
    Retr(String),
    Stor(String),
    Rfnr(String),
    Rnto(String),
    Mkd(String),
    Dele(String),
    Rmd(String),
    Type,
}


// la je vais faire un ennum pour la fonction connect_data_socket 
// comme ca ca sera plus simple deja vaec ennum et j'aurais une fonction 
// plus general
pub enum DataChannelFunction{
    SendListing(String),     
    SendFile(String),         
    ReceiveFile(String),
} 

impl FtpCommand {
 
    /// function called when LIST command is received 
    /// @param [path] &PathBuf object for the path of the dir 
    /// @return Result<String> a string within a Result
    pub fn ftp_command_list(path: &PathBuf) -> io::Result<String> {
        let mut result = String::new(); 
        let paths = fs::read_dir(path.as_path())?;

        for entry in paths {
            let entry: DirEntry = entry?; 
            let path = entry.path(); 
            let metadata = fs::metadata(path)?;

            let type_char = if metadata.is_dir() { "d" } else { "-" };
            let permissions = if metadata.is_dir() { "rwxr-xr-x" } else { "rw-r--r--" };
            let size = metadata.len();

            let name = entry.file_name();
            let clean_name = name.to_str().expect("Can't convert to str").trim_matches(|c| c == '"' || c == '\\' || c == '/');
            let size = metadata.len();
           
            let date_str = metadata.modified()
                .map(|t| DateTime::<Local>::from(t)) // get the correct format from the metadata
                // type (transform it (map it) from SystemTime to DateTime)
                .unwrap_or(Local::now()) // if it fails get current date 
                .format("%b %d %H:%M") 
                .to_string()
                .replace('\u{a0}', ""); 
           

            let final_str = format!(
                "{}{}    1 owner    group {} {} {}\r\n",
                type_char,
                permissions,
                size,
                date_str,
                clean_name
            );
            debug!("{:?}",final_str);
            result.push_str(&final_str);
        }      

        Ok(result)    
        
    }
    
    /// function called when CWD command is received 
    /// @param [new_path] the PathBuf object that represents the folder path 
    /// @return Classic Result 
    pub fn ftp_command_cwd(new_path: &PathBuf) -> Result<String, String> {
        

        // on verifie si ce chemin existe 
        if !new_path.exists(){
            return Err(FtpResponse::DIR_NOT_FOUND.to_string());
        }
        // si c un repertoire ou pas
        if !new_path.is_dir(){
            return Err(FtpResponse::NOT_A_DIR.to_string());
        }
        
        Ok(FtpResponse::DIR_CHANGED.to_string())    
    }
       
    /// function called when PWD command is received
    /// @param [path] the path 
    /// @return the response 
    pub fn ftp_command_pwd(path: &PathBuf) -> String {
        FtpResponse::current_dir(path)   
    }

    
    /// function called when RETR command is received 
    /// @param [path] the file/dir path to retrieve 
    /// @param [data_stream] the data socket 
    /// @return classic Result
    pub fn ftp_command_retr(path: &PathBuf, data_stream: &mut TcpStream) -> io::Result<()> {
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Not found"));
        }

        if path.is_file() {
            Self::ftp_command_retr_file(&path, data_stream)?;
        } else if path.is_dir() {
            Self::ftp_command_retr_dir(&path, data_stream)?;
        }

        data_stream.flush()?;
        Ok(())
    }

    /// function used to retrieve files in ftp_command_retr
    /// @param [path] the path of the file 
    /// @param [data_stream] the data socket 
    /// @return classic Result
    pub fn ftp_command_retr_file(path: &PathBuf, data_stream: &mut TcpStream) ->io::Result<()> {
        if !path.exists(){
            return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound, "File not found", // make own error 
            ));
        }

        if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,"Not a file",
        ));
        }

        let mut file = File::open(path)?;

        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;

            if bytes_read == 0 {
                break;  
            }

            data_stream.write_all(&buffer[..bytes_read])?;
        }

        data_stream.flush()?;
        Ok(())
    }
    
    /// function used to retrieve directories in ftp_command_retr
    /// @param [path] the path of the dir
    /// @param [data_stream] the data socket 
    /// @return classic Result
    fn ftp_command_retr_dir(dir_path: &Path, data_stream: &mut TcpStream) -> io::Result<()> {
        let dir_name = dir_path.file_name().unwrap_or_default().to_string_lossy();

        // on cree le nom de l'archive temporaire
        let tar_name = format!("{}.tar", dir_name);
        let tar_path = PathBuf::from(&tar_name);

        // on cree le fichier archive
        let tar_file = File::create(&tar_path)?;
        let mut builder = Builder::new(tar_file);

        // on ajoute recursivement le contenu du dossier dasn l'archive 
        builder.append_dir_all(dir_name.as_ref(), dir_path)?;
        builder.finish()?;

        drop(builder);

        // on envoie l'archive au client
        FtpCommand::ftp_command_retr_file(&tar_path, data_stream)?;

        // et on supprime l'archive temporraire
        fs::remove_file(&tar_path)?;

        Ok(())
    }

    /// function called when STOR command is received 
    /// @param [path] the file/dir path to store 
    /// @param [data_stream] the data socket 
    /// @return classic Result
    pub fn ftp_command_stor(path: &PathBuf, data_stream: &mut TcpStream) -> io::Result<()> {
        
        Self::ftp_command_stor_file(path, data_stream)?;
        // si c une archive on l'extrait
        if path.ends_with(".tar") {
            Self::ftp_command_stor_extract(path)?;
            fs::remove_file(path)?;  // on supprime l'archive après extraction
        }
        
        Ok(())
    }

    /// function used to store files in ftp_command_stor
    /// @param [path] the path of the file 
    /// @param [data_stream] the data socket 
    /// @return classic Result
    fn ftp_command_stor_file(path: &Path, data_stream: &mut TcpStream) -> io::Result<()> {
        let mut file = File::create(path)?;
        let mut buffer = [0; 4096];
        
        loop {
            let bytes_read = data_stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            file.write_all(&buffer[..bytes_read])?;
        }
        
        file.flush()?;
        Ok(())
    }

    /// function used to store directories in ftp_command_stor
    /// @param [path] the path of the dir
    /// @param [data_stream] the data socket 
    /// @return classic Result
    fn ftp_command_stor_extract(tar_path: &Path) -> io::Result<()> {
        let tar_file = File::open(tar_path)?;
        let mut archive = Archive::new(tar_file);
        
        // on l'extrait dans le dossier parent
        let dest_dir = tar_path.parent().unwrap_or(Path::new("."));
        archive.unpack(dest_dir)?;
        
        Ok(())
    }

    /// function used to parse the input command and then calls the appropriate function.
    /// @param [line] the input command 
    /// @return Option<FtpCommand> Maybe it's a command or it's Nothing.
    pub fn parse_ftp_command(line: &str) -> Option<FtpCommand>{
        let line = line.trim();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty(){
            return None;
        } 
        match parts[0]{
            "TYPE" => Some(FtpCommand::Type),
            
            "USER" => {
                if parts.len() >= 2 {
                    Some(FtpCommand::Usr(parts[1].to_string()))
                } else {
                    None
                }
            }

            "PASS" => {
                if parts.len() >= 2 {
                    Some(FtpCommand::Pass(parts[1].to_string()))
                } else {
                    None
                }
            } 
            
            "PWD" => Some(FtpCommand::Pwd),
            "LIST" => Some(FtpCommand::List),
            "CWD" => {
                if parts.len() >= 2 {
                    Some(FtpCommand::Cwd(parts[1].to_string()))
                } else {
                    None
                }
            }
            "RETR" => {
                if parts.len() >= 2 {
                    Some(FtpCommand::Retr(parts[1].to_string()))
                } else {
                    None
                }
            }
            "STOR" => { 
                if parts.len() >= 2 {
                    Some(FtpCommand::Stor(parts[1].to_string()))
                } else {
                    None
                }
            }
            "PASV" => Some(FtpCommand::Pasv),
            "RNFR" => Some(FtpCommand::Rfnr(parts[1].to_string())), 
            "RNTO" => Some(FtpCommand::Rnto(parts[1].to_string())),
            "MKD" => if parts.len() >= 2 {
                    Some(FtpCommand::Mkd(parts[1].to_string()))
                } else {None}
            "DELE" => if parts.len() >= 2 {
                    Some(FtpCommand::Dele(parts[1].to_string()))
                } else {None}
            "RMD" => if parts.len() >= 2 {
                    Some(FtpCommand::Rmd(parts[1].to_string()))
                } else {None}
            _ => None,
        }  
    } 
} 
