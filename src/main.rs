use std::{
    io::{ prelude::*, BufReader },
    net::{ TcpListener, TcpStream }, 
    fs,
};

const ROOT_DIR: &str = "html";
const AUTO_INDEX: bool = true;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8787").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let request_path: &str = &http_request[0].split(" ").collect::<Vec<&str>>()[1];

    match fs::read_to_string(ROOT_DIR.to_string() + request_path) {
        Ok(file) => {
            return serve_file(stream, file)
        },
        Err(error) => match error.raw_os_error().unwrap() {
            21 => { // Is a directory
                match AUTO_INDEX {
                    true => match fs::read_to_string(ROOT_DIR.to_string() + request_path + "index.html") {
                        Ok(file) => {
                            serve_file(stream, file)
                        },
                        Err(error) => match error.raw_os_error().unwrap() {
                            2 => { // Not found
                                return handle_404(stream)
                            },
                            other_error => {
                                println!("{:#?}", other_error)
                            }
                        }
                    },
                    false => {
                        return handle_404(stream)
                    }
                }
            },
            2 => { // Not found
                return handle_404(stream)
            },
            other_error => {
                println!("{:#?}", other_error)
            }
        }
    };
}

fn serve_file(mut stream: TcpStream, file: String) {
    let content_length = file.len();
    let mut response = format!("HTTP/1.1 200 OK
                           Content-Length: {length}
                           Content-Type: text/html \n\r\n\r", length = content_length);
    response += &file;

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_404(mut stream: TcpStream) {
    let file_404 = match fs::read_to_string(ROOT_DIR.to_string() + "/404.html") {
        Ok(file) => file,
        Err(error) => panic!("{}", error)
    };

    let content_length = file_404.len();

    let mut response = format!("HTTP/1.1 404 not found
                               Content-Length: {length}
                               Content-Type: text/html \n\r\n\r", length = content_length);

    response += &file_404;

    stream.write_all(response.as_bytes()).unwrap();
}

