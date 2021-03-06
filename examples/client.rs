extern crate futures;
extern crate hyper;
extern crate hyperlocal;
extern crate tokio;

use hyper::Body;
use std::io::{self, Write};

use futures::Stream;
use futures::Future;
use hyper::Client;
use hyperlocal::{Uri, UnixConnector};

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    HyperError(::hyper::Error),
}

impl From<std::io::Error> for ClientError {
    fn from(error: std::io::Error) -> Self {
        ClientError::IoError(error)
    }
}

impl From<::hyper::Error> for ClientError {
    fn from(error: ::hyper::Error) -> Self {
        ClientError::HyperError(error)
    }
}


fn run() -> () {
    let client = Client::builder()
        .build::<_, Body>(UnixConnector::new());

    let work = client
        .get(Uri::new("test.sock", "/").into())
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: \n{:?}", res.headers());

            res.into_body().for_each(|chunk| {
                io::stdout().write_all(&chunk).map_err(|e| panic!("example expects stdout is open, error={}", e))

            }).map_err(From::from)
        })
        .map(|_| {
            println!("\n\nDone.");
        }).map_err(|err| {
        println!("server error {:?}", err);
        });;

    tokio::run(work)
}

fn main() {
    run()
}
