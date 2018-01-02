
use hyper;
use hyper::server::{Http, Request, Response, Service};
use std::path::Path;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;

use tokio_core::reactor::Handle;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

pub trait ServerBaseExtend {
    fn call(&self, req: Request, handle: &Handle) -> Box<Future<Item=Response, Error=hyper::Error>>;
}

pub struct ServerBase<T: ServerBaseExtend> {
    tokio_handle: Handle,
    inner: T,
}

impl<T: ServerBaseExtend> Service for ServerBase<T> {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

        {
            let uri_path = Path::new(req.path());

            if !uri_path.is_absolute() {
                panic!("Tylko absolutne ścieżki są dozwolone");
            }
        }

        self.inner.call(req, &self.tokio_handle)
    }
}

impl<T: ServerBaseExtend + 'static> ServerBase<T> {
    pub fn run<FBuild>(srv_addr: SocketAddr, build: FBuild) where FBuild: Fn(&Handle) -> T {

        let http = Http::new();
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let listener = TcpListener::bind(&srv_addr, &handle).unwrap();
        let server = listener
            .incoming()
            .for_each(|(sock, addr)| {

                let hello_world = ServerBase {
                    tokio_handle: handle.clone(),
                    inner: build(&handle)
                };

                http.bind_connection(&handle, sock, addr, hello_world);
                Ok(())
            });

        core.run(server).unwrap();
    }
}

