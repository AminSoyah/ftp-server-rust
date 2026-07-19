use std::{
    sync::{Arc,atomic::{AtomicI32}},
    path::{Path,PathBuf},
}; 

pub struct UserSession {
    pub usr: String, 
    pub pwd: String, 
    pub current_dir: PathBuf,
    pub starting_dir: PathBuf, 
    pub pending_file: PathBuf, // file to be renammed (not to loose it through commands)
    pub s_id:i32, 
}

impl UserSession {

    pub fn init_id_dir(s_id: i32, dir: String) -> UserSession{
        UserSession { 
            usr: (String::new()), 
            pwd: (String::new()),
            current_dir: PathBuf::from("/"), 
            starting_dir: PathBuf::from(dir),
            s_id:s_id,
            pending_file: PathBuf::new(),
        }
    }
}
