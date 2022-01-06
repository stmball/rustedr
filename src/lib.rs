use std::error::Error;
use std::io;
use std::fs;
use std::fmt;

pub fn run(filename: &String) -> Result<(), Box<dyn Error>> {

    let contents = read_edr(&filename)?;
    if contents.len() < 2048 {
        Err(Box::new(ParseError::FileTooShort))
    } else {
        let header : &[u8] = &contents[..2048];
        let header_meta = parse_header(header);
        Ok(())
    }
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

fn parse_header(header: &[u8]) -> EDRMetadata {
    let ascii_header: String = header.to_vec().iter().map(|&x| x as char).collect();
    println!("{}", ascii_header);
    EDRMetadata { tane : 1}
}

struct EDRMetadata {
    tane: i32
}

#[derive(Debug, PartialEq)]
enum ReadError {
    EmptyFileName,
    ExtensionError,
    FileReadError(io::ErrorKind),
}

#[derive(Debug, PartialEq)]
enum ParseError {
    FileTooShort,
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::FileReadError(err.kind())
    }
}

impl Error for ReadError {}

impl Error for ParseError {}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::EmptyFileName => write!(f, "empty file name."),
            ReadError::ExtensionError => write!(f, "invalid file extension."),
            ReadError::FileReadError(err) => write!(f, "error reading file, {:?}", err)
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::FileTooShort => write!(f, "file contents too short."),
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

    #[test]
    fn header_too_short () {

    }
}

