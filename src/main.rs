#![feature(custom_attribute)] 

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate futures_cpupool;

use std::thread;
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
                println!("1. thread id={:?}", thread::current().id());

                Box::new(
                    self.cpu_pool.spawn_fn(move || {

                        println!("2. thread id={:?}", thread::current().id());

                        let file_content = file_read::read_file("./src/main.rs");

                        let mut response = Response::new();
                        response.set_body(file_content);
                        Ok(response)
                    })
                )
            }

            _ => {
                println!("3. thread id={:?}", thread::current().id());

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
