use std::net::{TcpStream, TcpListener};
use std::fs;
use std::io::{prelude::*, BufReader};

use rust_server::ThreadPool;

fn main() {
    // Addr : Port for server to listen to.
    let addr : &str = "127.0.0.1:8080";
    // Size : Size of thread pool.
    let size = 5;
    let ls : TcpListener = TcpListener::bind(addr).unwrap_or_else(|e| {
        panic!("Fail to set up TCP @ {}: {}", addr, e)
    });
    let pool = ThreadPool::new(size);
    for stream in ls.incoming() {
        let stream = stream.unwrap();
        pool.execute(| | {handle_connection(stream);});
    }
    println!("Closing server")
}

fn handle_connection(mut strm : TcpStream) {
    // Parse buffer -> terminate when buffer is empty.
    // TODO: Handle error in unwrapping results.
    let br = BufReader::new(&mut strm);
    let req_header =br.lines().next()
        .unwrap().unwrap_or_else(|e| {panic!("Problem parsing Request Header : {}", e)});
    let (status, file) = if req_header == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "./src/assets/demo_page.html")} else {("HTTP/1.1 404 NOT FOUND", "./src/assets/404.html")};
    let contents = fs::read_to_string(file)
        .unwrap_or_else(|e| {panic!("Issue loading page : {}", e)});
    let length = contents.len();
    let resp = format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");
    strm.write_all(resp.as_bytes()).unwrap();
    strm.flush().unwrap();
}