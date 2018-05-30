//! Hyper client bindings for unix domain sockets

use hyper::client::connect::Connected;
use hyper::client::connect::Connect;
use std::io;
use std::clone::Clone;

use futures::IntoFuture;
use futures::future::{self, FutureResult};
use hyper::Uri as HyperUri;
use tokio_core::reactor::Handle;
use tokio_service::Service;
use tokio_uds::UnixStream;
use futures::Future;
use hyper::client::connect::Destination;

use super::Uri;

const UNIX_SCHEME: &str = "unix";

/// A type which implements hyper's client connector interface
/// for unix domain sockets
///
/// `UnixConnector` instances assume uri's
/// constructued with `hyperlocal::Uri::new()` which produce uris with a `unix://`
/// scheme
///
/// # examples
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
/// extern crate tokio_core;
///
/// let core = tokio_core::reactor::Core::new().unwrap();
/// let client = hyper::Client::configure()
///    .connector(
///      hyperlocal::UnixConnector::new(core.handle())
///    )
///    .build(&core.handle());
/// ```
pub struct UnixConnector();

impl UnixConnector {
    pub fn new() -> Self {
        UnixConnector{}
    }
}

    impl Connect for UnixConnector {
        type Transport = UnixStream;
        type Error = io::Error;
        type Future = Box<Future<Item = (UnixStream, Connected), Error = io::Error> + 'static + Send >;


    fn connect(&self, destination: Destination) -> Self::Future {
        if destination.scheme() != UNIX_SCHEME {
            return Box::new(future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid uri {:?}", destination),
            )));
        }
        match Uri::socket_path_from_host(destination.host()) {
            Some(path) => Box::new(UnixStream::connect(path).map(|e| (e, Connected::new()))),
            _ => Box::new(future::err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid uri {:?}", destination),
            ))),
        }
    }
}

// impl Service for UnixConnector {
//     type Request = HyperUri;
//     type Response = UnixStream;
//     type Error = io::Error;
//     type Future = FutureResult<UnixStream, io::Error>;

//     fn call(&self, uri: HyperUri) -> Self::Future {
//         if uri.scheme() != Some(UNIX_SCHEME) {
//             return future::err(io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 format!("Invalid uri {}", uri),
//             ));
//         }
//         match Uri::socket_path(&uri) {
//             Some(path) => UnixStream::connect(path, &self.0).into_future(),
//             _ => future::err(io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 format!("Invalid uri {}", uri),
//             )),
//         }
//     }
// }

impl Clone for UnixConnector {
    fn clone(&self) -> Self {
        UnixConnector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_core::reactor::Core;

    #[test]
    fn connector_rejects_non_unix_uris() {
        let mut core = Core::new().unwrap();
        let connector = UnixConnector::new();
        let work = connector.connect(Destination {
            uri: "http://google.com".parse().unwrap()
        });
        assert!(core.run(work).is_err())
    }

    #[test]
    fn connector_rejects_hand_crafted_unix_uris() {
        let mut core = Core::new().unwrap();
        let connector = UnixConnector::new();
        let work = connector.connect(Destination {
            uri: "http://google.com".parse().unwrap()
        });

        assert!(core.run(work).is_err())
    }
}
