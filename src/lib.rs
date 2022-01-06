use std::error::Error;
use std::io;
use std::fs;
use std::fmt;
use std::num::ParseFloatError;
use std::num::ParseIntError;

use regex::Regex;

pub fn run(filename: &String) -> Result<(), Box<dyn Error>> {

    let contents = read_edr(&filename)?;
    let parsed_data = parse_data(contents);
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

fn parse_data(data: Vec<u8>) -> Result<(), ParseError> {
    if data.len() >= 2048 {
        // Read header
        let header = parse_header(&data[..2048]);
        Ok(())
    } else {
        Err(ParseError::FileTooShort)
    }
}

fn parse_header(header: &[u8]) -> Result<EDRMetadata, ParseError> {
    let ascii_header: String = header.to_vec().iter().map(|&x| x as char).collect();

    let mut data = EDRMetadata {
        AD: 0,
        ADCMAX: 0,
        DT: 0.0,
        YAGn: Vec::new(),
        YCFn: Vec::new(),
        YZn: Vec::new()
        
    };

    for line in ascii_header.lines() {
        if line.starts_with("AD") {
            data.AD = get_number(&line)?.parse::<u8>()?;
        } else if line.starts_with("ADCMAX") {
            data.ADCMAX = get_number(&line)?.parse::<u8>()?;
        } else if line.starts_with("DT") {
            data.DT = get_number(&line)?.parse::<f32>()?;
        } else if line.starts_with("YAG") {
            data.YAGn.push(get_number(&line)?.parse::<u8>()?);
        } else if line.starts_with("YCF") {
            data.YCFn.push(get_number(&line)?.parse::<f32>()?);
        } else if line.starts_with("YZ") {
            data.YZn.push(get_number(&line)?.parse::<u8>()?);
        } else {
            continue;
        }
    }
    Ok(data)
}

fn get_number(line: &str) -> Result<&str, ParseError> {
    let stringy = line.split('=').nth_back(0);
    match stringy {
        Some(a) => Ok(a),
        _ => Err(ParseError::Header)
    }
}


#[derive(Debug)]
struct EDRMetadata {
    AD: u8,
    ADCMAX: u8,
    DT: f32,
    YAGn: Vec<u8>,
    YCFn: Vec<f32>,
    YZn: Vec<u8>
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
    Header,
    FieldError,
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::FileReadError(err.kind())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> ParseError {
        ParseError::FieldError
    }
}

impl From<ParseFloatError> for ParseError{
    fn from(err: ParseFloatError) -> ParseError {
        ParseError::FieldError
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
            ParseError::Header => write!(f, "something went wrong parsing the header."),
            &ParseError::FieldError => write!(f, "one of the fields in the header couldn't be parsed.")
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
        let actual_error = parse_data(vec![1,2,3,4]);
        assert!(actual_error.is_err());
        assert_eq!(actual_error.unwrap_err(), ParseError::FileTooShort)
    }
}

