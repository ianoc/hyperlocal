extern crate hyper;
extern crate hyperlocal;
extern crate futures;
extern crate tokio_service;
use futures::{future, Future};
use std::io;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, StatusCode};


static INDEX: &'static [u8] = b"Try POST /echo";
type ResponseFuture = Box<Future<Item=Response<Body>, Error=io::Error> + Send>;

// Using service_fn_ok, we can convert this function into a `Service`.
fn echo(req: Request<Body>) -> ResponseFuture {
   match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::POST, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/echo") => {
            Box::new(future::ok(Response::new(req.into_body())))
        },
        _ => {
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(res))
        }
    }
}

fn run() -> Result<(), hyperlocal::server::BindError> {
    let path = "test.sock";
    let svr = hyperlocal::server::Http::new().bind(path, || service_fn(echo))?;
    println!("Listening on unix://{path} with 1 thread.", path = path);
    svr.run()?;
    Ok(())
}

fn main() {
    run().unwrap()
}
