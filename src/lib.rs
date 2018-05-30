//! hyperlocal provides [hyper](http://github.com/hyperium/hyper) client and server bindings
//! for [unix domain sockets](https://github.com/tokio-rs/tokio-uds)
//!
//! See the `hyperlocal::UnixConnector` docs for how to configure hyper clients and the `hyperlocal::server::Http` docs
//! for how to configure hyper servers
extern crate futures;
extern crate hyper;
extern crate tokio;
extern crate tokio_uds;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate hex;

use std::borrow::Cow;
use std::path::Path;

use hyper::Uri as HyperUri;
use hex::{FromHex, ToHex};

pub mod server;
pub mod client;
pub use client::UnixConnector;

/// A type which implements `Into` for hyper's  `hyper::Uri` type
/// targetting unix domain sockets.
///
/// You can use this with any of
/// the HTTP factory methods on hyper's Client interface
/// and for creating requests
///
/// ```no_run
/// extern crate hyper;
/// extern crate hyperlocal;
///
/// let url = hyperlocal::Uri::new(
///   "/path/to/socket", "/urlpath?key=value"
///  );
///  let req: hyper::Request<hyper::Body> =
///    hyper::Request::new(
///      hyper::Get,
///      url.into()
///    );
/// ```
#[derive(Debug)]
pub struct Uri<'a> {
    /// url path including leading slash, path, and query string
    encoded: Cow<'a, str>,
}

impl<'a> Into<HyperUri> for Uri<'a> {
    fn into(self) -> HyperUri {
        self.encoded.as_ref().parse().unwrap()
    }
}

impl<'a> Uri<'a> {
    /// Productes a new `Uri` from path to domain socket and request path.
    /// request path should include a leading slash
    pub fn new<P>(socket: P, path: &'a str) -> Self
    where
        P: AsRef<Path>,
    {
        let host = socket.as_ref().to_string_lossy().as_bytes().to_hex();
        let host_str = format!("unix://{}:0{}", host, path);
        Uri { encoded: Cow::Owned(host_str) }
    }

    // fixme: would like to just use hyper::Result and hyper::error::UriError here
    // but UriError its not exposed for external use
    fn socket_path(uri: &HyperUri) -> Option<String> {
        uri.host()
            .iter()
            .filter_map(|host| {
                Vec::from_hex(host).ok().map(|raw| {
                    String::from_utf8_lossy(&raw).into_owned()
                })
            })
            .next()
    }

    fn socket_path_from_host(host: &str) -> Option<String> {
        Some(host)
            .iter()
            .filter_map(|host| {
                Vec::from_hex(host).ok().map(|raw| {
                    String::from_utf8_lossy(&raw).into_owned()
                })
            })
            .next()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Uri as HyperUri;

    #[test]
    fn unix_uris_into_hyper_uris() {
        let unix: HyperUri = Uri::new("foo.sock", "/").into();
        let expected: HyperUri = "unix://666f6f2e736f636b:0/".parse().unwrap();
        assert_eq!(unix, expected);
    }

    #[test]
    fn unix_uris_resolve_socket_path() {
        let unix: HyperUri = "unix://666f6f2e736f636b:0/".parse().unwrap();
        let path = Uri::socket_path(&unix).unwrap();
        let expected = "foo.sock";
        assert_eq!(path, expected);
    }
}

