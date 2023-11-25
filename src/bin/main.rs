use std::{
    env::current_dir,
    fs,
    io::{prelude::*, ErrorKind},
    net::{TcpListener, TcpStream},
};
use core::panic;
use glob::glob;
use relative_path::RelativePath;
use server::ThreadPool;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for flux in listener.incoming() {
        let flux = flux.unwrap();
        pool.exec(|| connection_manager(flux));
    }
}


fn connection_manager(mut stream: TcpStream) {
    let mut tempon = [0; 1024];
    stream.read(&mut tempon).unwrap_or_else(|e| {
        println!("{e:?}");
        panic!("Une erreur s'est produite !")
    });
    let request_string = String::from_utf8_lossy(&tempon[..]);
    let request_header: &str = match request_string.split("\r\n").next() {
        Some(value) => value,
        None => return,
    };
    let request_header: Vec<&str> = request_header.split(" ").collect();

    let uri = match request_header.get(1) {
        Some(value) => value,
        None => return,
    };
    let response = build_response(uri);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
/// function that build the response 
fn build_response(uri: &str) -> String {
    let root = current_dir().unwrap();

    println!("{root:?}");
    let path = RelativePath::new(uri).to_path(&root);
    let content;
    if path.is_dir() {
        let mut dir_content = String::new();
        for element in glob(&format!("{}/*", path.as_os_str().to_str().unwrap()))
            .expect("Failed to read glob pattern")
        {
            match element {
                Ok(entry) => {
                    dir_content.push_str("<div class=\"dir\">");
                    dir_content.push_str(&entry.display().to_string());
                    dir_content.push_str("</div>");
                }
                Err(e) => println!("{:?}", e),
            }
        }
        content = str::replace(
            &fs::read_to_string("dir.html").unwrap(),
            "{dir_content}",
            &dir_content,
        );
        println!("Client request for a folder");
    } else if path.is_file() {
        content = fs::read_to_string(path).unwrap_or_else(|e| {
            if e.kind() == ErrorKind::InvalidData {
                String::from("<h1>Impossible d'ouvrir un ficher binaire</h1>")
            } else {
                panic!()
            }
        });
    } else {
        content = fs::read_to_string("404.html").unwrap();
    }
    let header = "HTTP/1.1 200 OK \r\n";
    format!(
        "{header} Content-Length:{}\r\n\r\n {content}",
        content.len()
    )
}


