use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, BufReader};
use std::fs;

fn main() {
    let addr : &str = "127.0.0.1:8080";
    let ls : TcpListener = TcpListener::bind(addr).unwrap_or_else(|e| {
        panic!("Fail to set up TCP @ {}: {}", addr, e)
    });
    for stream in ls.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut strm : TcpStream) {
    // Parse buffer -> terminate when buffer is empty.
    // TODO: Handle error in unwrapping results.
    let br = BufReader::new(&mut strm);
    let req_header =br.lines().next()
        .unwrap().unwrap_or_else(|e| {panic!("Problem parsing Request Header : {}", e)});
    let (status, file) = if req_header == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "./assets/demo_page.html")} else {("HTTP/1.1 404 NOT FOUND", "./assets/404.html")};
    let contents = fs::read_to_string(file)
        .unwrap_or_else(|e| {panic!("Issue loading page : {}", e)});
    let length = contents.len();
    let resp = format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");
    strm.write_all(resp.as_bytes()).unwrap();
}
