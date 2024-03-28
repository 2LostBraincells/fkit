use std::error::Error;

pub fn current_directory_name() -> Result<String, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let dir_name = current_dir
        .file_name()
        .ok_or("Could not get the directory name")?
        .to_str()
        .ok_or("Could not convert directory name to a string")?;

    Ok(dir_name.to_string())
}
