use gcloud::{
    datastore::{DatastoreClient, ToEntity},
    AuthProvider,
};

const SK_PATH: &str = "/home/wduss/.google/dalloriam-dev.json";

#[derive(Debug, ToEntity)]
struct Person {
    name: String,
    age: i32,
}

fn main() {
    let auth = AuthProvider::from_json_file(SK_PATH).unwrap();
    let mut client = DatastoreClient::new(String::from("shift3-dev"), auth);

    let p = Person {
        name: String::from("John Smith"),
        age: 22,
    };

    let b = Person {
        name: String::from("Jane Doe"),
        age: 22,
    };

    client.insert(p).unwrap();
    client.insert(b).unwrap();
    client.commit().unwrap();

    let persons = client.get_all::<Person>().unwrap();
    println!("{:?}", persons);
}
