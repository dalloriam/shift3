use google_pubsub1::Pubsub;

use yup_oauth2::{ApplicationSecret, Authenticator, DefaultAuthenticatorDelegate, MemoryStorage};

fn new_client() -> hyper::Client {
    let tls_client = hyper_rustls::TlsClient::new();
    let https_connector = hyper::net::HttpsConnector::new(tls_client);

    hyper::Client::with_connector(https_connector)
}

pub struct PubSubClient {
    hub: Pubsub<hyper::Client, yup_oauth2::ServiceAccountAccess<hyper::Client>>,

    project_id: String,
}

impl PubSubClient {
    pub fn new(project_id: &str) -> PubSubClient {
        let secret =
            yup_oauth2::service_account_key_from_file(&String::from("service_account.json"))
                .unwrap();

        let access = yup_oauth2::ServiceAccountAccess::new(secret, new_client());
        let hub = Pubsub::new(new_client(), access);

        PubSubClient {
            hub,
            project_id: String::from(project_id),
        }
    }

    pub fn publish_message(&self, body: &str, topic: &str) {
        let message = google_pubsub1::PubsubMessage {
            data: Some(base64::encode("bing bong".as_bytes())),
            ..Default::default()
        };

        let request = google_pubsub1::PublishRequest {
            messages: Some(vec![message]),
        };
        let (resp, pub_resp) = self
            .hub
            .projects()
            .topics_publish(request, &format!("{}/topics/{}", self.project_id, topic))
            .doit()
            .unwrap();
    }

    pub fn pull_message(&self, topic: &str) {
        let request = google_pubsub1::PullRequest {
            return_immediately: Some(false),
            max_messages: Some(1),
        };

        let (rsp, pull_rsp) = self
            .hub
            .projects()
            .subscriptions_pull(
                request,
                &format!("{}/subscriptions/{}", self.project_id, topic),
            )
            .doit()
            .unwrap();
        for msg in pull_rsp.received_messages.unwrap().into_iter() {
            let data = msg.message.unwrap().data.unwrap();
            let decoded = base64::decode(&data).unwrap();
            let out_msg = String::from_utf8_lossy(&decoded);
            println!("Got message: {}", out_msg);
        }
    }
}

fn main() {
    let client = PubSubClient::new("projects/personal-workspace");
    client.publish_message("bing bong", "shift3");
    client.pull_message("shift3-consume")
}
