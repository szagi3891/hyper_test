#![feature(custom_attribute)] 

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate futures_cpupool;

use futures_cpupool::CpuPool;
use futures::future::Future;
use hyper::Client;

use std::net::SocketAddr;

use hyper::{Method, StatusCode};
//use hyper::header::ContentLength;
use hyper::server::{Request, Response};
use hyper::header::ContentType;

use tokio_core::reactor::Handle;
use std::path::{Path};

mod file_read;
mod server_template;
mod static_file;
mod match_str;

use server_template::{ServerBase, ServerBaseExtend};
use static_file::StaticFile;

//https://gist.github.com/meganehouser/d5e1b47eb2873797ebdc440b0ed482df

//Static : https://github.com/stephank/hyper-staticfile/blob/master/examples/doc_server.rs

fn set_header(response: &mut Response, path: &str) {
    let ext = match Path::new(path).extension() {
        Some(ext) => match ext.to_str() {
            Some("txt")  => ContentType::plaintext() ,
            Some("jpg")  => ContentType::jpeg(),
            Some("png")  => ContentType::png(),
            //Some("html") => Type::TextHtml,
            Some(_)      => ContentType::html(),
            None         => ContentType::html(),
        },
        
        None => ContentType::html(),
    };

    let headers = response.headers_mut();
    headers.set(ext);
}

struct HelloWorldServer {
    cpu_pool: CpuPool,
    static_file: StaticFile,
}

impl ServerBaseExtend for HelloWorldServer {
    fn call(&self, req: Request, handle: &Handle) -> Box<Future<Item=Response, Error=hyper::Error>> {

        println!("Request {}", req.path());

        if req.method() == &Method::Get {
            let req_path = req.path();
            if let Some(rest) = match_str::match_str(req_path, "/static/") {
                match self.static_file.to_response(rest) {
                    Ok(mut response) => {
                        set_header(&mut response, rest);
                        return Box::new(futures::future::ok(response));
                    },
                    Err(_err) => {
                        let mut resp = Response::new()
                            .with_status(StatusCode::NotFound);
                        return Box::new(futures::future::ok(resp));
                    }
                }
            }
        }

        match (req.method(), req.path()) {

            (&Method::Get, "/specjalny") => {
                let mut response = Response::new();
                response.set_body("Try POSTing data to /echo");
                Box::new(futures::future::ok(response))
            }

            (&Method::Get, "/form") => {
                let mut response = Response::new();
                response.set_body("<form action='/submit'><input text='data' /></form>");
                Box::new(futures::future::ok(response))
            }

            (&Method::Post, "/echo") => {
                let mut response = Response::new();
                response.set_body(req.body());
                Box::new(futures::future::ok(response))
            }

            (&Method::Get, "/crawl") => {
                let uri = "http://muzyka.onet.pl/rock/spin-ranking-najlepszych-plyt-25-lecia/cgf71".parse().unwrap();

                Box::new(
                    Client::new(&handle)
                        .get(uri)
                        .and_then(|res| {

                            println!("Response: {}", res.status());

                            let mut response = Response::new();
                            response.set_body(res.body());
                            futures::future::ok(response)
                        })
                )
            }
            (&Method::Get, "/crash") => {
                panic!("crash");
            }

            (&Method::Get, "/file") => {
                Box::new(
                    self.cpu_pool.spawn_fn(move || {
                        let file_content = file_read::read_file("./src/main.rs");

                        let mut response = Response::new();
                        response.set_body(file_content);
                        Ok(response)
                    })
                )
            }

            _ => {
                const PHRASE: &'static str = "Hellllllloooooo";

                Box::new(futures::future::ok(
                    Response::new()
                        .with_status(StatusCode::NotFound)
                        //.with_header(ContentLength(PHRASE.len() as u64))
                        .with_body(PHRASE)
                ))
            }
        }
    }
}

fn main() {
    let addr = "127.0.0.1:7777";
    let srv_addr: SocketAddr = addr.parse().unwrap();
    
    println!("server start {}", addr);

    let cpu_pool = CpuPool::new_num_cpus();
    let cpu_pool_file = CpuPool::new(16);

    ServerBase::run(srv_addr, |handle: &Handle| {
        HelloWorldServer {
            cpu_pool: cpu_pool.clone(),
            static_file: StaticFile::new(handle.clone(), Path::new("./static_public"), cpu_pool_file.clone()),
        }
    });
}
