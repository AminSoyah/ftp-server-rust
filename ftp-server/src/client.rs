use std::{
    net::{TcpStream, TcpListener},
    io::{BufWriter, Result, Error, Write, BufRead, BufReader,Read},
    path::{Path, PathBuf},
    fs::{File}
};
use std::fs;
use std::io;
use tar::{Builder, Archive};
use crate::{
    commands::{DataChannelFunction, FtpCommand}, errors::{ErrorsData, FtpResult}, ftp_responses as FtpResponse, listener, user_session::UserSession
}; 


use log::{debug, error, log_enabled, info, Level};

pub struct FtpClient{ // to hold references struct needs to implement a lifetime 
    session: UserSession,
    data_listener: Option<TcpListener>,
    writer_buf: BufWriter<TcpStream>,
    reader_buf: BufReader<TcpStream>, 
} 

pub trait Client { 
    // let that for the mfs MOCKS
}
    
impl FtpClient{

    pub fn new(usr_s: UserSession, stream:TcpStream) -> FtpClient{
        FtpClient {
            session: usr_s, 
            reader_buf: BufReader::new(stream.try_clone().expect("Stream not initialized")), 
            writer_buf: BufWriter::new(stream), 
            data_listener: None,

        }
    }
    
    /// for resolve_any_path & resolve_path : 
    ///     user current_dir is starting with "/" instead of "/starting_dir"
    ///     this is to prevent user to access data above the starting_dir
    ///     but for the server, need to rebuild the absolute path, by adding "starting_dir" to the relative user path
    ///     
    ///
    /// resolve_path does not take parameter, only used for LIST because LIST display current
    /// directory
    /// @param [self]
    /// @return the absolute path of the user current_dir
    pub fn resolve_path(&self) -> PathBuf {
        let mut path = self.session.starting_dir.clone(); 
        if let Ok(user_path) = self.session.current_dir.strip_prefix("/") {
            path.push(user_path);
        }
        path
    }

    /// same as above but takes a parameter : the future file/dir path 
    /// @param [self]
    /// @param [virtual_path] the future relative path 
    /// @return the future absolute path
    pub fn resolve_any_path(&self, virtual_path: &PathBuf) -> PathBuf {
        let mut path = self.session.starting_dir.clone();
        if let Ok(user_path) = virtual_path.strip_prefix("/") {
            path.push(user_path);
        } else {
            path.push(virtual_path);
        }
        path
    }

    pub fn absolute_or_relative(&self, filename: String) -> PathBuf{
        if filename.starts_with("/") {
            PathBuf::from(&filename)
        } else {
            let mut p = self.session.current_dir.clone();
            p.push(&filename);
            p
        }
    }

    
    

    /// Methods firstly written by Amin -- Check commands.rs deletions if needed 
    /// handle_connection | handle_command | handle_data_transfer | connect_data_socket | setup_passive_mode
    /// I added some stuff tho (such as buffers & debug)


    /// function called when server receive a client @see 'listener.rs'
    /// infinite loop 
    /// calls FtpCommand::{parse_ftp_command}
    ///       FtpResponse::{internal_error, UNKNOWN_CMD}
    /// @param [self] 
    /// @return FtpResult @see 'errros.rs' (type of Error) 
    pub fn handle_connection(&mut self) -> FtpResult<()> {
        
        self.writer_buf.write_all(FtpResponse::WELCOME.as_bytes())?; 
        self.writer_buf.flush()?;

        let mut line = String::new(); 

        loop {
            line.clear(); 

            let bytes_read = self.reader_buf.read_line(&mut line)?; 
            if (bytes_read == 0){break;}

            if let Some(cmd) = FtpCommand::parse_ftp_command(&line){
                if let Err(e) = self.handle_command(cmd) {
                    debug!("> Error received during handle_command : {:?}",e); 
                    self.writer_buf.write_all(FtpResponse::internal_error(&e.to_string()).as_bytes())?;
                    self.writer_buf.flush()?; 
                } 
            } else {
                info!("> Failed while parsing command");
                debug!("> (handle_connection) FAILED while parsing");
                self.writer_buf.write_all(FtpResponse::UNKNOWN_CMD.as_bytes())?;
                self.writer_buf.flush()?;
            }
            
            self.writer_buf.flush()?; 
        }
        Ok(())
    }

    
    /// function called by handle_command 
    /// it opens a passive connection 
    /// @param [self] 
    /// @return FtpResult, type of Result 
    pub fn setup_passive_mode(&mut self) -> FtpResult<(String)> {
    
        // Ouvre un port aléatoire pour le canal de données
        self.data_listener = Some(TcpListener::bind("127.0.0.1:0")?);  
        
        // si je mest pour listenenr port 0 faut que je mette la ligne en dessous 
        // le 0 en gros c pour choisir un port aleatoire
        let addr = match self.data_listener.as_ref() {
            Some(listener) => listener.local_addr()?, 
            None => {
                debug!("> Error while opening data channel");
                self.writer_buf.write_all(FtpResponse::DATA_CONNECTION_FAILED.as_bytes())?; 
                return Err(ErrorsData::DataConnectionError);
            }
        }; 
        
        // ca du coup affiche le port 
        info!("Data channel openned on port : {}\n", addr.port()); 

        // ici on calcule p1 et p2 pour la réponse PASV
        let port = addr.port();
        let p1 = port / 256;
        let p2 = port % 256;
        
        // pour formater la réponse FTP
        let response = FtpResponse::pasv_mode(p1, p2);
        
        Ok((response))
    }
    
    /// function called by handle_connection
    /// it matches the parsed command and executes the correct function 
    /// uses (FtpCommand::{allEnum, setup_passive_mode, ftp_command_list,prepare_message_to_client }
    ///      (self::{handle_data_transfer},
    ///      (FtpResponse::{failed_pasv, FILE_NOT_FOUND}
    /// @param [self]
    /// @param [cmd](FtpCommand) the command received 
    /// @return standard result Ok(())
    pub fn handle_command(&mut self, cmd: FtpCommand) -> Result<()> {        
        debug!("> (handle_command) RECEIVED {:?}",cmd);
        match cmd {
            
            FtpCommand::Usr(username) => {
                self.session.usr = username.clone(); 
                let response = FtpResponse::user_ok(&username);
                self.writer_buf.write_all(response.as_bytes())?;
                self.writer_buf.flush()?;
            }

            FtpCommand::Pass(pass) => {
                self.session.pwd = pass.clone(); 
                let response = FtpResponse::LOGIN_SUCCESS.to_string();
                self.writer_buf.write_all(response.as_bytes())?;
                self.writer_buf.flush()?; 
            }

            FtpCommand::Pasv => {
                info!("> Pasv cmd sent");
                match self.setup_passive_mode() {
                    Ok((response)) => {
                        self.writer_buf.write_all(response.as_bytes())?;
                        self.writer_buf.flush()?;
                        
                        // on stocke le listener pour les commandes LIST/RETR/STOR
                        // je l'ai move dans la fonction au dessus 
                        info!(">,passif mode activated");
                    }
                    Err(e) => {
                        // Si erreur le port est deja utilise par exemple
                        self.writer_buf.write_all(FtpResponse::failed_pasv(&e.to_string()).as_bytes())?;
                        self.writer_buf.flush()?;
                    }
                }
            }

            FtpCommand::List => {
                let listing = FtpCommand::ftp_command_list(&self.resolve_path())?;
                self.handle_data_transfer(DataChannelFunction::SendListing(listing))?;
            }    
            
            FtpCommand::Retr(filename) => {
                
                let path = self.absolute_or_relative(filename);                 
                let actual_path = self.resolve_any_path(&path);
                
                if !actual_path.exists() {
                    self.writer_buf.write_all(FtpResponse::FILE_NOT_FOUND.as_bytes())?;
                    self.writer_buf.flush()?;
                    return Ok(());
                }
                
                // puis on convertir pathBuf en strung pour le transfert
                let path_str = actual_path.to_string_lossy().to_string();
                
                self.handle_data_transfer(DataChannelFunction::SendFile(path_str))?;
            }

            FtpCommand::Stor(filename) => {
                
                let path = self.absolute_or_relative(filename); 
                
                let actual_path = self.resolve_any_path(&path);
                
                
                // puis on convertir pathBuf en strung pour le transfert
                let path_str = actual_path.to_string_lossy().to_string(); 
                self.handle_data_transfer(DataChannelFunction::ReceiveFile(path_str))?;
            }

            FtpCommand::Pwd => { 
                let response = FtpCommand::ftp_command_pwd(&self.session.current_dir);
                self.writer_buf.write_all(response.as_bytes());
                self.writer_buf.flush()?;
            }

            FtpCommand::Cwd(new_dir) => {
                let path = self.absolute_or_relative(new_dir); 
                let actual_path = self.resolve_any_path(&path);

                let response = match FtpCommand::ftp_command_cwd(&actual_path) {
                    Ok(dir_changed) => {
                        self.session.current_dir = path;
                        dir_changed
                    }
                    Err(err_m) => {
                        debug!("> (cwd) error : {:?}",err_m);
                        err_m
                    }
                };
                self.writer_buf.write_all(response.as_bytes()); 
                self.writer_buf.flush()?;

            }
            FtpCommand::Type => {
                self.writer_buf.write_all("200 Type set OK.\r\n".to_string().as_bytes());
                self.writer_buf.flush()?;
            }

            FtpCommand::Rfnr(old_file) => {
                
                let file_path = self.absolute_or_relative(old_file); 

                let path = self.resolve_any_path(&file_path);

                debug!("path: {:?}",path);
                if (path.exists()) {
                    self.writer_buf.write_all(FtpResponse::RFNR.to_string().as_bytes()); 
                    self.session.pending_file = path ; 
                } 
                else {self.writer_buf.write_all(FtpResponse::FILE_NOT_FOUND.to_string().as_bytes());}
                self.writer_buf.flush()?;
            }

            FtpCommand::Rnto(new_file) => {
                if (self.session.pending_file != PathBuf::new()) {
                    let file_path = self.absolute_or_relative(new_file); 
                    let path = self.resolve_any_path(&PathBuf::from(&file_path)); 

                    match fs::rename(self.session.pending_file.clone(), path) {
                        Ok(_) => {
                            self.writer_buf.write_all(FtpResponse::FILE_RENAMED.to_string().as_bytes());
                        } 
                        Err(_) => {
                            self.writer_buf.write_all(FtpResponse::RNTO_ERROR.to_string().as_bytes());
                        }
                    }
                }
            }

            FtpCommand::Mkd(dirname) => { 
                let dir = self.absolute_or_relative(dirname); 
                let path = self.resolve_any_path(&dir); 
                match fs::create_dir(path) {
                    Ok(_) => self.writer_buf.write_all(FtpResponse::mkd_ok(dir.to_str().expect("Dir creation error")).to_string().as_bytes()), 
                    Err(_) => self.writer_buf.write_all(FtpResponse::DIR_ALREADY_EXISTS.to_string().as_bytes()),
                };
                self.writer_buf.flush()?;

            }

            FtpCommand::Dele(name) => {
                let path_name = self.absolute_or_relative(name); 
                let path = self.resolve_any_path(&path_name); 

                if (path.exists()) {
                    match fs::remove_file(path) {
                        Ok(_) => self.writer_buf.write_all(FtpResponse::FILE_DELETED.to_string().as_bytes()),
                        Err(_) => self.writer_buf.write_all(FtpResponse::FILE_NOT_FOUND.to_string().as_bytes()),
                    };
                    self.writer_buf.flush()?;
                }
            }

            FtpCommand::Rmd(name) => {
                let path_name = self.absolute_or_relative(name); 
                let path = self.resolve_any_path(&path_name); 

                if (path.exists()) {
                    match fs::remove_dir(path) {
                        Ok(_) => self.writer_buf.write_all(FtpResponse::mkd_del(path_name.to_str().expect("Dir deletion error")).to_string().as_bytes()),
                        Err(_) => self.writer_buf.write_all(FtpResponse::DIR_NOT_FOUND.to_string().as_bytes()),
                    };
                    self.writer_buf.flush()?;
                }               
            } 
        }
    
        Ok(())  
    }



    /// function called by handle_data_transfer 
    /// it uses the freshly initialized data socket to perform wanted actions
    /// calls ftp_command_retr and ftp_command_stor and ftp_command_list (through
    /// SendListing(listing) tho)
    /// @param [self] 
    /// @param [cmd] The DataChannelFunction which will be called
    /// @return Result 
    pub fn connect_data_socket(&mut self, cmd: DataChannelFunction) -> Result<()>{
        
        // le data_listener est un listenner pour le canal de données on va le recuperer 
        // grace a la focntion setup_passive_mode
        info!("> wait for data channel connexion ");
        let (mut stream,_) = self.data_listener.as_ref().expect("DataChanel not ready ").accept()?;
        info!("> data channel connected");

        match cmd{
            DataChannelFunction::SendListing(listing) => {
                stream.write_all(listing.as_bytes())?;
            }
            DataChannelFunction::SendFile(filename) => {
                let path = PathBuf::from(filename);
                FtpCommand::ftp_command_retr(&path, &mut stream)?;
            }
            DataChannelFunction::ReceiveFile(filename) => {
                let path = PathBuf::from(filename);
                FtpCommand::ftp_command_stor(&path, &mut stream)?;
            }
        }
        stream.flush()?;
        drop(stream);
        info!("> data channel closed");

        Ok(())
    }

    /// function called by handle_command above 
    /// it prepares data socket , send the message and write the answer
    /// it finally sets the data_listener to None as we won't use it again
    /// @param [self] 
    /// @param [operation] the DataChannelFunction which will be called 
    /// @return Result 
    pub fn handle_data_transfer(&mut self, operation: DataChannelFunction) -> Result<()> {
        
        if self.data_listener.is_none() {
            debug!("> data_listener was none | please use PASV first");
            &self.writer_buf.write_all(FtpResponse::USE_PASV_FIRST.as_bytes())?;
            return Ok(());
        }
        
        // On envoie  150 pour pour l'ouverture du data channel
        &self.writer_buf.write_all(FtpResponse::OPENING_DATA.as_bytes())?;
        &self.writer_buf.flush()?;
        
        // puis executer l'une des commande appelée
        if let Some(listener) = &self.data_listener {
            self.connect_data_socket(operation)?;
        }
        
        // Envoie 226 pour la fin du transfert
        &self.writer_buf.write_all(FtpResponse::TRANSFER_COMPLETE.as_bytes())?;
        self.data_listener = None;
        // en parametre
        
        Ok(())
    }

}

impl Client for FtpClient {



}
