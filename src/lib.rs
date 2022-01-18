use std::convert::TryInto;
use std::error::Error;
use std::fmt::format;
use std::io;
use std::fs;
use std::fmt;
use std::io::Write;
use std::num::ParseFloatError;
use std::num::ParseIntError;

pub fn run(filename: &String) -> Result<(), Box<dyn Error>> {

    // Read the file
    let contents = read_edr(&filename)?;
    // Parse the file's header and contents
    let parsed_data = parse_data(contents)?;

    let written_data = write_data(parsed_data, filename)?;

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

fn parse_data(data: Vec<u8>) -> Result<Vec<Vec<f32>>, ParseError> {
    if data.len() > 2048 {
        // Read header
        let header = parse_header(&data[..2048])?;

        // Read contents
        let contents = parse_contents(&data[2048..], header)?;

        Ok(contents)

    } else if data.len() == 2048 {
        Err(ParseError::FileOnlyHeader)
    } else {
        Err(ParseError::FileTooShort)
    }
}

fn write_data(data: Vec<Vec<f32>>, filename: &String) -> Result<(), WriteError>{

    let new_filename = format!("{}.csv", filename.strip_suffix(".EDR").unwrap());
    let mut file = fs::File::create(new_filename)?;
    for line in transpose(data) {
        write!(file, "{}\n", line.iter()
                                .map(|k| k.to_string())
                                .collect::<Vec<String>>()
                                .join(","))?
        }
    Ok(())
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> where T: Clone {
    assert!(!v.is_empty());
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
        .collect()
}

fn parse_header(header: &[u8]) -> Result<EDRMetadata, ParseError> {
    let ascii_header: String = header.to_vec().iter().map(|&x| x as char).collect();

    let mut metadata = EDRMetadata {
        AD: 0,
        ADCMAX: 0,
        DT: 0.0,
        YAGn: Vec::new(),
        YCFn: Vec::new(),
        YZn: Vec::new()
        
    };

    for line in ascii_header.lines() {
        if line.starts_with("AD") {
            metadata.AD = get_number(&line)?.parse::<i32>()?;
        } else if line.starts_with("ADCMAX") {
            metadata.ADCMAX = get_number(&line)?.parse::<i32>()?;
        } else if line.starts_with("DT") {
            metadata.DT = get_number(&line)?.parse::<f32>()?;
        } else if line.starts_with("YAG") {
            metadata.YAGn.push(get_number(&line)?.parse::<i32>()?);
        } else if line.starts_with("YCF") {
            metadata.YCFn.push(get_number(&line)?.parse::<f32>()?);
        } else if line.starts_with("YZ") {
            metadata.YZn.push(get_number(&line)?.parse::<i32>()?);
        } else {
            continue;
        }
    }
    Ok(metadata)
}

fn parse_contents(data: &[u8], metadata: EDRMetadata) -> Result<Vec<Vec<f32>>, ParseError> {
    
    let no_channels = metadata.YZn.len();
    
    println!("{}", no_channels);
    let contents = data.chunks_exact(2).map(|x| i16::from_le_bytes(x.try_into().unwrap())).collect::<Vec<i16>>();
    
    let data = (0..no_channels)
                .map(|x| contents
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % no_channels == x)
                        .map(|(_, j)| scale_val(*j as f32, metadata.YZn[x], metadata.AD, metadata.YCFn[x], metadata.YAGn[x], metadata.ADCMAX))
                        .collect::<Vec<f32>>())
                        .collect::<Vec<Vec<f32>>>();
    Ok(data)

}

fn scale_val(value: f32, yz: i32, ad: i32, ycf: f32, yag: i32, adcmax: i32) -> f32 {
    return ((value - yz as f32) * ad as f32) / (ycf * yag as f32 * (adcmax as f32 + 1.0)) 
}

fn get_number(line: &str) -> Result<&str, ParseError> {
    let stringy = line.split('=').nth_back(0);
    match stringy {
        Some(a) => Ok(a),
        _ => Err(ParseError::HeaderTagNotFound)
    }
}


#[derive(Debug)]
struct EDRMetadata {
    AD: i32,
    ADCMAX: i32,
    DT: f32,
    YAGn: Vec<i32>,
    YCFn: Vec<f32>,
    YZn: Vec<i32>
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
    FileOnlyHeader,
    HeaderTagNotFound,
    FieldError,
}

#[derive(Debug, PartialEq)]
enum WriteError {
    FileWriteError(io::ErrorKind),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::FileReadError(err.kind())
    }
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> WriteError {
        WriteError::FileWriteError(err.kind())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> ParseError {
        ParseError::FieldError
    }
}

impl From<ParseFloatError> for ParseError{
    fn from(_: ParseFloatError) -> ParseError {
        ParseError::FieldError
    }
} 

impl Error for ReadError {}

impl Error for ParseError {}

impl Error for WriteError {}

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
            ParseError::FileOnlyHeader => write!(f, "file contains only a header with no data."),
            ParseError::HeaderTagNotFound => write!(f, "something went wrong parsing the header."),
            ParseError::FieldError => write!(f, "one of the fields in the header couldn't be parsed.")
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::FileWriteError(err) => write!(f, "error writing file, {:?}", err),
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

