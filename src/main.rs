extern crate hyper;
extern crate futures;

use futures::future::Future;

use hyper::{Method, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

const PHRASE: &'static str = "Hellllllloooooo";

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

struct HelloWorld;

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
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
    println!("Server start on port 3000 ....");

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();
    server.run().unwrap();
}
