use thiserror::Error ;
use std::{
    error::Error, 
    io::{BufWriter, IntoInnerError},
    net::TcpStream
};

#[derive (Error, Debug)]
pub enum ErrorsData {
    #[error("Error while connecting to server")]
    ServerConnectionError(#[from] std::io::Error), 

    #[error("Error while handling client connection")]
    ClientConnectionError,

    #[error("Error while unwrapping stream")]
    StreamUnwrapError(#[from] IntoInnerError<BufWriter<TcpStream>>),

    #[error("Error while opening Data chanel")]
    DataConnectionError,
}

pub type FtpResult<T> = std::result::Result<T, ErrorsData>; 
