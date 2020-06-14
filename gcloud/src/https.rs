use hyper::{net::HttpsConnector, Client};
use hyper_rustls::TlsClient;

/// Returns a preconfigured Hyper TLS client.
///
/// The client uses a hyper HttpsConnector with a `hyper_rustls` TLS Client.
pub fn new_tls_client() -> Client {
    let tls_client = TlsClient::new();
    let https_connector = HttpsConnector::new(tls_client);
    Client::with_connector(https_connector)
}

#[cfg(test)]
mod tests {
    use super::new_tls_client;

    #[test]
    fn test_client_instantiation() {
        // Not much to test..
        // Let's validate that client instatiation doesn't panic at least..
        let _client = new_tls_client();
    }
}
