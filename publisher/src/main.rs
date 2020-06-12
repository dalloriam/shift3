use std::collections::HashMap;

use google_datastore1::{
    BeginTransactionRequest, CommitRequest, Datastore, Entity, Key, Mutation, PathElement,
};
use yup_oauth2::{ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate, MemoryStorage};

fn new_client() -> hyper::Client {
    let tls_client = hyper_rustls::TlsClient::new();
    let https_connector = hyper::net::HttpsConnector::new(tls_client);

    hyper::Client::with_connector(https_connector)
}

fn main() {
    let secret =
        yup_oauth2::service_account_key_from_file(&String::from("service_account.json")).unwrap();
    let access = yup_oauth2::ServiceAccountAccess::new(secret, new_client());

    let hub = Datastore::new(new_client(), access);

    let m = Mutation {
        insert: Some(Entity {
            key: Some(Key {
                path: Some(vec![PathElement {
                    kind: Some(String::from("Shift3-TriggerConfig")),
                    ..Default::default()
                }]),
                partition_id: None,
            }),
            properties: Some(HashMap::new()),
        }),
        ..Default::default()
    };

    let commit_req = CommitRequest {
        mode: Some(String::from("NON_TRANSACTIONAL")),
        transaction: None,
        mutations: Some(vec![m]),
    };

    let (r, resp) = hub
        .projects()
        .commit(commit_req, "personal-workspace")
        .doit()
        .unwrap();
}
