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

use rocket::http::Status;
use rocket::response::Failure;
use rocket::Data;
use std::fmt::Write;
use std::io::Read;
use std::result::Result;
use std::str;

use db::*;

#[get("/")]
fn index() -> &'static str {
    "
    USAGE:

        POST /

            Accepts public PGP key in the request body and responds with URL of page containing public PGP key

        GET /<fingerprint>

            Retrieves the 40-byte public PGP key metadata with fingerprint `<fingerprint>`
    "
}

#[post("/", data = "<data>")]
fn upload(data: Data, connection: DbConn) -> Result<String, Failure> {
    let mut key_string = String::new();

    if data.open().read_to_string(&mut key_string).is_err() {
        return Err(Failure(Status::BadRequest));
    };

    let tpk = match openpgp::TPK::from_reader(armored!(key_string)) {
        Ok(tpk) => tpk,
        Err(_) => {
            return Err(Failure(Status::BadRequest));
        }
    };

    let mut tpk_serialized = Vec::new();
    if tpk.serialize(&mut tpk_serialized).is_err() {
        return Err(Failure(Status::InternalServerError));
    };

    let pgpkey = Key {
        fingerprint: tpk.fingerprint().as_slice().to_vec(),
        pgpkey: tpk_serialized,
    };

    match db::insert(pgpkey, &connection) {
        Ok(key) => Ok(["/key/", hex::encode(key.fingerprint).as_str()].concat()),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}

#[get("/key/<fingerprint>")]
fn retrieve(fingerprint: String, connection: DbConn) -> Result<String, Failure> {
    if fingerprint.len() != 40 {
        return Err(Failure(Status::BadRequest));
    }

    let fingerprint_bytes = match hex::decode(fingerprint) {
        Ok(b) => b,
        Err(_) => {
            return Err(Failure(Status::BadRequest));
        }
    };

    let pgpkey: Vec<u8> = match db::get(fingerprint_bytes, &connection) {
        Ok(key) => key.pgpkey,
        Err(diesel::result::Error::NotFound) => {
            return Err(Failure(Status::NotFound));
        }
        Err(_) => {
            return Err(Failure(Status::InternalServerError));
        }
    };

    let tpk = match openpgp::TPK::from_bytes(&pgpkey) {
        Ok(tpk) => tpk,
        Err(_) => {
            return Err(Failure(Status::InternalServerError));
        }
    };

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
        assert_eq!(response.body_string(), Some(
        "
    USAGE:

        POST /

            Accepts public PGP key in the request body and responds with URL of page containing public PGP key

        GET /<fingerprint>

            Retrieves the 40-byte public PGP key metadata with fingerprint `<fingerprint>`
    ".into()));
    }

    #[test]
    fn upload_test_success() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        db::_delete(
            hex::decode(_UPLOAD_TEST_FINGERPRINT).unwrap(),
            &init_pool().get().unwrap(),
        ).unwrap();

        let mut response = client.post("/").body(_UPLOAD_TEST_KEY).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(_UPLOAD_TEST_URL.to_string()));

        db::_delete(
            hex::decode(_UPLOAD_TEST_FINGERPRINT).unwrap(),
            &init_pool().get().unwrap(),
        ).unwrap();
    }

    /*
    #[test]
    fn upload_test_bad_key() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        db::delete(
            _UPLOAD_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();

        let mut response = client.post("/").body(_UPLOAD_TEST_KEY[..60]).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(_UPLOAD_TEST_URL.to_string()));

        db::delete(
            _UPLOAD_TEST_FINGERPRINT.to_string(),
            &init_pool().get().unwrap(),
        ).unwrap();
    }
    */

    #[test]
    fn retrieve_test_success() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        db::_delete(
            hex::decode(_RETRIEVE_TEST_FINGERPRINT).unwrap(),
            &init_pool().get().unwrap(),
        ).unwrap();

        let mut response = client.post("/").body(_RETRIEVE_TEST_KEY).dispatch();

        let mut response = client
            .get(URI::new(response.body_string().unwrap()))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string(),
            Some(_RETRIEVE_TEST_BODY.to_string())
        );

        db::_delete(
            hex::decode(_RETRIEVE_TEST_FINGERPRINT).unwrap(),
            &init_pool().get().unwrap(),
        ).unwrap();
    }

    #[test]
    fn retrieve_test_bad_key() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        let response = client
            .get(URI::new("/key/notarealkeyforobviousreasons"))
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn retrieve_test_no_key() {
        let client = Client::new(
            rocket::ignite()
                .manage(init_pool())
                .mount("/", routes![index, upload, retrieve]),
        ).expect("valid rocket instance");

        let response = client
            .get(URI::new("/key/123456789ABCDEF123456789ABCDEF123456789A"))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }
}
