#![cfg_attr(feature = "cargo-clippy", warn(clippy_pedantic))]
#![feature(plugin, decl_macro, extern_prelude)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate openpgp;
extern crate hex;

mod consts;
mod db;
mod schema;

use rocket::Data;
use std::fmt::Write;
use std::io;
use std::io::Read;
use std::str;

use db::*;

#[get("/")]
fn index() -> &'static str {
    "
    USAGE:

    POST /

          Accepts public PGP key in the request body and responds with URL of page containing public PGP key

    GET /<fingerprint>

          retrieves the 40-byte public PGP key with fingerprint `<fingerprint>`
    "
}

#[post("/", data = "<data>")]
fn upload(data: Data, connection: DbConn) -> io::Result<String> {
    let mut key_string = String::new();
    data.open().read_to_string(&mut key_string).unwrap();

    let tpk = openpgp::TPK::from_reader(armored!(key_string)).unwrap();

    // Check if already exists
    // If it does check signatures

    let mut tpk_serialized = Vec::new();
    tpk.serialize(&mut tpk_serialized).unwrap();

    let pgpkey = Key {
        fingerprint: tpk.fingerprint().to_hex(),
        pgpkey: hex::encode(tpk_serialized),
    };

    db::insert(pgpkey, &connection).unwrap();

    Ok(["/key/", tpk.fingerprint().to_hex().as_str()].concat())
}

#[get("/key/<fingerprint>")]
fn retrieve(fingerprint: String, connection: DbConn) -> io::Result<String> {
    let pgpkey_hex = db::get(fingerprint, &connection).unwrap().pgpkey;

    let tpk = openpgp::TPK::from_bytes(&hex::decode(pgpkey_hex).unwrap()).unwrap();

    let mut key_output = String::new();

    writeln!(&mut key_output, "Fingerprint: {}", tpk.fingerprint()).unwrap();
    writeln!(&mut key_output).unwrap();

    for (i, u) in tpk.userids().enumerate() {
        writeln!(
            &mut key_output,
            "{}: UID: {}, {} self-signature(s), {} certification(s)",
            i,
            u.userid(),
            u.selfsigs().count(),
            u.certifications().count()
        ).unwrap();
    }

    writeln!(&mut key_output).unwrap();

    for (i, s) in tpk.subkeys().enumerate() {
        writeln!(
            &mut key_output,
            "{}: Fingerprint: {}, {} self-signature(s), {} certification(s)",
            i,
            s.subkey().fingerprint(),
            s.selfsigs().count(),
            s.certifications().count()
        ).unwrap();
    }

    Ok(key_output)
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, upload, retrieve])
        .launch();
}

#[cfg(test)]
mod test {
    use super::*;
    use consts::*;
    use rocket::http::uri::URI;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn index_test() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        let mut response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("
    USAGE:

    POST /

          Accepts public PGP key in the request body and responds with URL of page containing public PGP key

    GET /<fingerprint>

          retrieves the 40-byte public PGP key with fingerprint `<fingerprint>`
    ".into()));
    }

    #[test]
    fn upload_test() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        db::delete(
            _UPLOAD_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();

        let mut response = client.post("/").body(_UPLOAD_TEST_KEY).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(_UPLOAD_TEST_URL.to_string()));

        db::delete(
            _UPLOAD_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();
    }

    #[test]
    fn retrieve_test() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        db::delete(
            _RETRIEVE_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();

        let mut response = client.post("/").body(_RETRIEVE_TEST_KEY).dispatch();

        let mut response = client
            .get(URI::new(response.body_string().unwrap()))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(_RETRIEVE_TEST_BODY.to_string()));

        db::delete(
            _RETRIEVE_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();
    }

}
