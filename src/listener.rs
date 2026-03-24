use std::{
    io::{Result, Error, BufWriter, Write}, // io::Result is forcing the error to be type of io::Error
    // tale std::result::Result for the Result<T,Err> type
    option::{Option},
    net::{TcpListener, TcpStream, IpAddr, Ipv4Addr},
    sync::{Arc,Mutex, atomic::{AtomicI32,Ordering}},
    collections::{HashMap}

};

use log::{debug, error, log_enabled, info, Level};

use crate::{
    client::{Client, FtpClient}, errors::{ErrorsData, FtpResult}, user_session::UserSession
};

pub struct ServerListener{
    //listener: Option<TcpListener>,
    ip_v4: String,
    t_pool: Arc<rayon::ThreadPool>, // t(hread)Pool 
    g_id : Arc<AtomicI32>, // unique id generator (it does not decrement when client disconnect but resets for each ServerListener
    start_dir: String // will be passed while creating the user session, which is not in thread so no need of arc pointers
}


impl ServerListener{

    pub fn new(ip : String, port: i32, nb_thread: usize, start_dir: String) -> ServerListener{
        
        debug!("ServerListener created with ip {} and port {}", ip, port); 
        ServerListener {
            //listener : None, 
            ip_v4: format!("{}:{}",ip,port),  
            t_pool : Arc::new(rayon::ThreadPoolBuilder::new().num_threads(nb_thread).build().unwrap()), 
            g_id: Arc::new(AtomicI32::new(0)),  
            start_dir: start_dir,

        }
    }

        
    // @func : loop waiting for client connection 
    //          incoming is basically a loop{accept()} 
    // @param : [self] (you really need a description?)
    // @return: FtpResult<TcpListner> see{crate::errors} 
    //          type for std::result::Result<T, ErrorsData>;
    //
    pub fn listen_clients(self: &Arc<Self>) -> FtpResult<TcpListener>{
        let binding = TcpListener::bind(self.ip_v4.clone())?; 
        
        for stream in binding.incoming(){
            match stream { // stream is Result<TcpStream, Err> 
                Ok(s) => {
                    let g_id_c = Arc::clone(&self.g_id); 
                    let u_id = g_id_c.fetch_add(1, Ordering::SeqCst) ; // return prev val and add 1  
                    let pool_c = Arc::clone(&self.t_pool) ;
                    let self_c = Arc::clone(&self);
                    pool_c.spawn(move ||{
                        match self_c.handle_client_connect(s,u_id)  { 
                            Ok(c) => {
                                info!("> client connected to server"); 
                            } 
                            Err(e) => error!("> client connection failed"),
                        }
                    })
                }
                Err(e) => return Err(ErrorsData::ServerConnectionError(e)), 
            }
        }

        Ok(binding)  
    }


    // @func : called in a thread when client connects 
    // @param : [stream] TcpStream object received into incoming() 
    // @param : [u_id] unique id generated thread-safely 
    // @return : FtpResult<TcpListner> see{crate::errors} 
    //          type for std::result::Result<T, ErrorsData>;
    pub fn handle_client_connect(&self, stream: TcpStream, u_id: i32 ) -> FtpResult<FtpClient> { 
        
        let user_s = UserSession::init_id_dir(u_id, self.start_dir.clone());
        let mut clt = FtpClient::new(user_s,stream);
        

        clt.handle_connection(); 

        Ok(clt) 
    }
}


