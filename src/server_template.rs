
use hyper;
use hyper::server::{Http, Request, Response, Service};

use std::net::SocketAddr;
use futures_cpupool::CpuPool;

use futures::Future;
use futures::stream::Stream;

use tokio_core::reactor::Handle as TokioHandle;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

pub struct Context {
    pub tokio_handle: TokioHandle,
    pub cpu_pool: CpuPool,
}

pub trait ServerBaseExtend {
    fn call(&self, req: Request, context: Context) -> Box<Future<Item=Response, Error=hyper::Error>>;
}

pub struct ServerBase<T: ServerBaseExtend> {
    tokio_handle: TokioHandle,
    cpu_pool: CpuPool,
    inner: T,
}

impl<T: ServerBaseExtend> Service for ServerBase<T> {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        self.inner.call(req, Context {
            cpu_pool: self.cpu_pool.clone(),
            tokio_handle: self.tokio_handle.clone(),
        })
    }
}

impl<T: ServerBaseExtend + Clone + 'static> ServerBase<T> {
    pub fn run(srv_addr: SocketAddr, inner: T) {
        let cpu_pool = CpuPool::new_num_cpus();

        let http = Http::new();
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let listener = TcpListener::bind(&srv_addr, &handle).unwrap();
        let server = listener
            .incoming()
            .for_each(|(sock, addr)| {

                let hello_world = ServerBase {
                    tokio_handle: handle.clone(),
                    cpu_pool: cpu_pool.clone(),
                    inner: inner.clone()
                };

                http.bind_connection(&handle, sock, addr, hello_world);
                Ok(())
            });

        core.run(server).unwrap();
    }
}

