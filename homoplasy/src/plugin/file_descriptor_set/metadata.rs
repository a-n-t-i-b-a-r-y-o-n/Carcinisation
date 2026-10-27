use anyhow::{anyhow, Result};

/// Parse the syntax field in a proto file
pub(super) fn parse_syntax<'t, T>(tokens: &mut T) -> Result<String>
where
    T: Iterator<Item = &'t str>
{
    // Equals sign
    _ = tokens.next();

    // Syntax string in quotes with a trailing semicolon
    match tokens.next() {
        Some(next) => {
            let token = next.trim_end_matches(';')
                .trim_matches('"')
                .trim_matches('\'');

            Ok(String::from(token))
        }
        None => Err(anyhow!("Failed to parse protobuf syntax"))
    }
}

/// Parse the package name in a proto file
pub(super) fn parse_package<'t, T>(tokens: &mut T) -> Result<String>
where
    T: Iterator<Item = &'t str>
{
    // Package name with a trailing semicolon
    match tokens.next() {
        Some(next) => {
            let token = next.trim_end_matches(';');

            Ok(String::from(token))
        }
        None => Err(anyhow!("Failed to parse protobuf package name"))
    }
}