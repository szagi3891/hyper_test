#![feature(custom_attribute)] 

extern crate hyper;
extern crate futures;
extern crate hyper_staticfile;
extern crate tokio_core;
extern crate futures_cpupool;

use std::thread;
use futures::future::Future;
use hyper::Client;
use std::path::Path;

use std::net::SocketAddr;

use hyper::{Method, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Request, Response, Service};
use hyper_staticfile::Static;

use tokio_core::reactor::Handle as TokioHandle;

const PHRASE: &'static str = "Hellllllloooooo";

mod file_read;
mod server_template;

use server_template::{ServerBase, ServerBaseExtend, Context};

//https://gist.github.com/meganehouser/d5e1b47eb2873797ebdc440b0ed482df

//Static : https://github.com/stephank/hyper-staticfile/blob/master/examples/doc_server.rs

fn match_str<'a>(data: &'a str, pattern: &'a str) -> Option<&'a str> {
    let pattern_len = pattern.len();

    if data.len() >= pattern_len {
        let (head, body) = data.split_at(pattern_len);
        if head == pattern {
            return Some(body);
        }
    }

    None
}

#[test]
fn match_str_test() {
    let aaa = "abcdef";

    assert_eq!(match_str(aaa, ""), Some("abcdef"));
    assert_eq!(match_str(aaa, "abc"), Some("def"));
    assert_eq!(match_str(aaa, "abcde"), Some("f"));
    assert_eq!(match_str(aaa, "abcdef"), Some(""));
    assert_eq!(match_str(aaa, "abd"), None);
    assert_eq!(match_str(aaa, "abdeffffff"), None);
    assert_eq!(match_str(aaa, "ffff"), None);
}

struct HelloWorld {
    http_static: Static,
}

impl ServerBaseExtend for HelloWorld {
    fn call(&self, req: Request, context: Context) -> Box<Future<Item=Response, Error=hyper::Error>> {

        println!("Request {}", req.path());

        let to_run = {
            let req_path = req.path();
            match_str(req_path, "/static/").is_some()
        };

        if to_run {
            return self.http_static.call(req);
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
                    Client::new(&context.tokio_handle)
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
                    context.cpu_pool.spawn_fn(move || {

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

    ServerBase::run(srv_addr, |handle: &TokioHandle| {
        HelloWorld {
            http_static: Static::new(&handle, Path::new("/static")),
        }
    });
}
