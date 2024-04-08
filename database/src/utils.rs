const ALLOWED_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";

/// Encodes a string to be safe for use in a SQL query
/// 
/// Only characters in the set [A-Za-z0-9_] are allowed
/// any other characters are excluded from the human-readable part
///
/// # Example
///
/// ```
/// # use database::utils::sql_encode;
/// let output = sql_encode("Hello, world!");
/// assert_eq!(output, Err("Helloworld".to_string()));
/// ```
///
/// ```
/// # use database::utils::sql_encode;
/// let output = sql_encode("Hello_world");
/// assert_eq!(output, Ok("Hello_world".to_string()));
/// ```
pub fn sql_encode(input: &str) -> Result<String,String> {
    let mut output = String::with_capacity(input.len());
    let mut safe = true;

    for c in input.chars() {
        if ALLOWED_CHARS.contains(c) {
            output.push(c);
        } else {
            safe = false;
        }
    }

    match safe {
        true => Ok(output),
        false => Err(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_encode() {
        let output = sql_encode("Hello, world!");
        assert!(output.is_err());

        assert_eq!(output.unwrap_err(), "Helloworld".to_string());
    }

    #[test]
    fn test_sql_encode_safe() {
        let output = sql_encode("Hello_world");

        assert_eq!(output.unwrap(), "Hello_world".to_string());
    }
}
