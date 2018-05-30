//! Hyper server bindings for unix domain sockets

use hyper::body::Payload;
use hyper::Body;
use std::path::Path;
use std;

use futures::{Future, Stream};

use hyper::server::conn::Http as HyperHttp;
use tokio_core::reactor::Core;
use hyper::service::NewService;
use tokio_uds::UnixListener;
use tokio;

/// An instance of a server created through `Http::bind`.
//
/// This structure is used to create instances of Servers to spawn off tasks
/// which handle a connection to an HTTP server.
pub struct Server<S>
{
    protocol: HyperHttp,
    new_service: S,
    core: Core,
    listener: UnixListener,
}

impl<S, Bd> Server<S>
where
    S: NewService<ReqBody=Body, ResBody=Bd> + Send + 'static,
    S::Error: Into<Box<::std::error::Error + Send + Sync>>,
    S::Service: Send,
    S::Error: Into<Box<::std::error::Error + Send + Sync>>,
    <S as ::hyper::service::NewService>::Future: Send,
    <S::Service as ::hyper::service::Service>::Future: Send + 'static,
    Bd: Payload
{
    pub fn run(self) -> ::hyper::Result<()> {
        let Server {
            protocol,
            new_service,
            mut core,
            listener,
            ..
        } = self;

        let _protocol = protocol.clone();
        let server = listener
            .incoming()
            .for_each(move |sock| {
                let _protocol = protocol.clone();

                let fut = new_service.new_service().map_err(|_| ()).map (move |service| {
                    _protocol.serve_connection(sock, service).map_err(|_| ())
                }).flatten();

                tokio::spawn(fut.map_err(|_| ()));
                Ok(())
            }).map_err(|_| ());

        core.run(server).unwrap();

        Ok(())
    }
}

/// A type that provides a factory interface for creating
/// unix socket based Servers
///
/// # examples
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// //let server = hyperlocal::Http::new().bind(
///  // "path/to/socket",
///  //  || Ok(HelloWorld)
/// //)
///
/// ```
pub struct Http {
}

impl Clone for Http {
    fn clone(&self) -> Http {
        Http { ..*self }
    }
}

#[derive(Debug)]
pub enum BindError {
    IoError(std::io::Error),
    HyperError(::hyper::Error),
}

impl From<std::io::Error> for BindError {
    fn from(error: std::io::Error) -> Self {
        BindError::IoError(error)
    }
}

impl From<::hyper::Error> for BindError {
    fn from(error: ::hyper::Error) -> Self {
        BindError::HyperError(error)
    }
}


impl Http {
    /// Creates a new instance of the HTTP protocol, ready to spawn a server or
    /// start accepting connections.
    pub fn new() -> Http {
        Http {  }
    }

    /// binds a new server instance to a unix domain socket path
    pub fn bind<P, S, Bd>(&self, path: P, new_service: S) -> Result<Server<S>, BindError>
    where
        P: AsRef<Path>,
        S: NewService<ReqBody = Body, ResBody= Bd>
            + Send
            + Sync
            + 'static,
        Bd: Payload,
    {
        let core = Core::new()?;
        let handle = core.handle();
        let listener = UnixListener::bind(path.as_ref())?;

        Ok(Server {
            protocol: HyperHttp::new(),
            new_service: new_service,
            core: core,
            listener: listener,
        })
    }
}
