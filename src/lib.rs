use std::error::Error;
use std::io;
use std::fs;
use std::fmt;

pub fn run(filename: &String) -> Result<(), Box<dyn Error>> {

    let contents = read_edr(&filename)?;

    Ok(())
}

fn read_edr(filename: &String) -> Result<Vec<u8>, ReadError> {
    // Check if the file has the correct extension.
    if filename.is_empty() {
        Err(ReadError::EmptyFileName)
    } else if !filename.ends_with(".EDR") {
        Err(ReadError::ExtensionError)
    } else {
        Ok(fs::read(filename)?)
    }
}

#[derive(Debug, PartialEq)]
enum ReadError {
    EmptyFileName,
    ExtensionError,
    FileReadError(io::ErrorKind),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::FileReadError(err.kind())
    }
}

impl std::error::Error for ReadError {}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::EmptyFileName => write!(f, "empty file name."),
            ReadError::ExtensionError => write!(f, "invalid file extension."),
            ReadError::FileReadError(err) => write!(f, "error reading file, {:?}", err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_name(){
        let actual_error = read_edr(&String::from(""));
        assert!(actual_error.is_err());
        assert_eq!(actual_error.unwrap_err(), ReadError::EmptyFileName);
    }

    #[test]
    fn incorrect_file_extension() {
        let actual_error = read_edr(&String::from("not_an_edr.txt"));
        assert!(actual_error.is_err());
        assert_eq!(actual_error.unwrap_err(), ReadError::ExtensionError);
    }

    #[test]
    fn file_does_not_exist () {
        let actual_error = read_edr(&String::from("non_existant.EDR"));
        assert!(actual_error.is_err());
        assert_eq!(actual_error.unwrap_err(), ReadError::FileReadError(io::ErrorKind::NotFound))
    }
}

