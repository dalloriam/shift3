use gcloud::{
    datastore::{DatastoreClient, ToEntity},
    AuthProvider,
};

const SK_PATH: &str = "/home/wduss/.google/dalloriam-dev.json";

#[derive(ToEntity)]
struct Person {
    name: String,
    age: i32,
}

fn main() {
    let auth = AuthProvider::from_json_file(SK_PATH).unwrap();
    let mut client = DatastoreClient::new(String::from("shift3-dev"), auth);

    for _i in 0..100 {
        let p = Person {
            name: String::from("John Smith"),
            age: 22,
        };

        client.insert(p).unwrap();
    }

    client.commit().unwrap();
}
