use std::fs;
use std::error::Error;
use std::path::PathBuf;

pub fn get_sqlite_path() -> Result<String, Box<dyn Error>> {
    let home_dir = match dirs::config_dir() {
        Some(path) => path.join("nats_aws_files"),
        None => return Err("Failed to find config directory".into()),
    };
    
    if !home_dir.exists() {
        fs::create_dir(&home_dir)?;
    }
    
    let db_path = home_dir.join("sled_db");
    
    // Convert the PathBuf to a String
    let db_path_str = db_path.to_string_lossy().into_owned();
    
    Ok(db_path_str)
}