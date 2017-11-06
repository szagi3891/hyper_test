#![feature(custom_attribute)] 

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate futures_cpupool;

//use std::sync::mpsc::channel;
//use std::thread;
//use std::time::Duration;
use std::net::SocketAddr;
use futures::future::Future;
use futures_cpupool::CpuPool;
use hyper::Client;

use hyper::{Method, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use futures::stream::Stream;

use tokio_core::reactor::Handle as TokioHandle;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

const PHRASE: &'static str = "Hellllllloooooo";

mod file_read;
//use file_read;

//https://gist.github.com/meganehouser/d5e1b47eb2873797ebdc440b0ed482df

struct HelloWorld {
    tokio_handle: TokioHandle,
    cpu_pool: CpuPool,
}

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

        println!("Rwquest {}", req.path());

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
                    Client::new(&self.tokio_handle)
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
                let file_content = file_read::read_file("./src/main.rs");

                println!("aaaa {}", file_content);

                let mut response = Response::new();
                response.set_body(file_content);
                Box::new(futures::future::ok(response))
            }

            _ => {
                Box::new(futures::future::ok(
                    Response::new()
                        .with_status(StatusCode::NotFound)
                        .with_header(ContentLength(PHRASE.len() as u64))
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

    let http = Http::new();
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let listener = TcpListener::bind(&srv_addr, &handle).unwrap();
    let server = listener
        .incoming()
        .for_each(|(sock, addr)| {

            let hello_world = HelloWorld {
                tokio_handle: handle.clone(),
                cpu_pool: cpu_pool.clone()
            };

            http.bind_connection(&handle, sock, addr, hello_world);
            Ok(())
        });

    core.run(server).unwrap();
}


/*
fn to_uppercase(chunk: Chunk) -> Chunk {
    let uppered = chunk.iter()
        .map(|byte| byte.to_ascii_uppercase())
        .collect::<Vec<u8>>();
    Chunk::from(uppered)
}

            (&Method::Post, "/echo") => {
                let mapping = req.body().map(to_uppercase as fn(Chunk) -> Chunk);
                let body: Box<Stream<Item=_, Error=_>> = Box::new(mapping);
                response.set_body(body);
            },
*/

/*
fn reverse(chunk: Chunk) -> Response {
    let reversed = chunk.iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>();
    Response::new()
        .with_body(reversed)
}

            (&Method::Post, "/echo") => {
                Box::new(
                    req.body()
                        .concat2()
                        .map(reverse)
                )
            },
*/

