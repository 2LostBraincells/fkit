use std::error::Error;

/// Gets the name of the current directory the binary was run from.
/// If youa re in a directory named `fkit` this function will return the string `"fkit"`
pub fn current_directory_name() -> Result<String, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let dir_name = current_dir
        .file_name()
        .ok_or("Could not get the directory name")?
        .to_str()
        .ok_or("Could not convert directory name to a string")?;

    Ok(dir_name.to_string())
}
