use std::net::{TcpStream, TcpListener};
use std::io::{self, Read, Write};
use std::fs::{self, OpenOptions};
use std::str;
use std::collections::HashMap;

const GET_REQUEST: &str = "GET / HTTP/1.1";
const POST_REQUEST: &str = "POST / HTTP/1.1";
// Bad request currently has no body and no headers might want to update later for a more "proper response"
// Also see  CRLF sequences so I don't forget later
const BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST\r\n\r\n";
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";


fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => handle_connection(stream)?,
            Err(error) => panic!("Couldn't handle connection error: {error}")
        }
    }

    return Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0u8; 2024];
    stream.read(&mut buffer)?;

    buffer
        .iter()
        .for_each(|byte| print!("{}", *byte as char));

    if buffer.starts_with(GET_REQUEST.as_bytes()) {
        println!("{:?}", parse_request_body(&buffer).unwrap())
    }
    else if buffer.starts_with(POST_REQUEST.as_bytes()) {
        todo!("Parse request and pass the prefix and content to table_write() or potentially table update in the future")
    } 
    else {
        stream.write(BAD_REQUEST.as_bytes())?;
    } 
    
    stream.write(OK_RESPONSE.as_bytes())?;
    stream.flush()?;

    return Ok(())
}

fn table_get(prefix: &str) -> io::Result<Option<String>> {
    let file_contents = fs::read_to_string("Data.txt")?;

    for line in file_contents.lines() {
        if line.starts_with(prefix) {
            return Ok(Some(line[prefix.len() + 1..].to_string()))
        }
    }
 
    return Ok(None)
}

fn table_write(prefix: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("Data.txt")?;

    let mut buffer = String::new();
    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents)?;

    let mut found_line = false;
    for line in file_contents.lines() {
        match line.starts_with(prefix) {
            false => buffer.push_str(line),
            true =>  {
                buffer.push_str(format!("{}:{}", prefix, content).as_str());
                found_line = true;
            }
        }
    }

    if !found_line { 
        buffer.push_str(format!("{}:{}", prefix, content).as_str());
    }

    file.write(buffer.as_bytes())?;

    return Ok(())
    
}

fn parse_request_body(request: &[u8]) -> Option<HashMap<&str, &str>> {
    let (body_start, body_end) = match (
        request.windows(4).position(|bytes| bytes == b"\r\n\r\n"),
        request.iter().position(|&byte| byte == b"\0"[0])
    ) {
        (Some(start), Some(end)) => (start + "\r\n\r\n".len(), end),
        _ => return None
    };
    // Convert those bytes into a string cause doing this with the &[u8] type looks really messy
    let body = str::from_utf8(&request[body_start..body_end])
        .expect("Request contained invalid UTF-8");

    let mut map = HashMap::new();
    for pair in body.split('&') {
        let mut iter = pair.split('=');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            map.insert(key, value);
        }
    }

    match map.len() {
        0 => return None,
        _ => return Some(map)
    }
}