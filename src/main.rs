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

    write!(&mut key_output, "Fingerprint: {}\n\n", tpk.fingerprint()).unwrap();

    for (i, u) in tpk.userids().enumerate() {
        write!(
            &mut key_output,
            "{}: UID: {}, {} self-signature(s), {} certification(s)\n",
            i,
            u.userid(),
            u.selfsigs().count(),
            u.certifications().count()
        ).unwrap();
    }

    write!(&mut key_output, "\n").unwrap();

    for (i, s) in tpk.subkeys().enumerate() {
        write!(
            &mut key_output,
            "{}: Fingerprint: {}, {} self-signature(s), {} certification(s)\n",
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
    use rocket::http::Status;
    use rocket::local::Client;
    use rocket::http::uri::URI;
    use diesel::PgConnection;

    #[test]
    fn index_test() {
        let client = Client::new(rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, upload, retrieve])
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
        let client = Client::new(rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, upload, retrieve])
        ).expect("valid rocket instance");

        db::delete("6C6FB8F67B4E5B64C77C42673995B95EE169A19B".to_string(), &init_pool().get().unwrap()).unwrap();

        let mut response = client.post("/").body("-----BEGIN PGP PUBLIC KEY BLOCK-----

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
=DoxB
-----END PGP PUBLIC KEY BLOCK-----").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("/key/6C6FB8F67B4E5B64C77C42673995B95EE169A19B".into()));

        db::delete("6C6FB8F67B4E5B64C77C42673995B95EE169A19B".to_string(), &init_pool().get().unwrap()).unwrap();
    }

    #[test]
    fn retreive_test() {
        let client = Client::new(rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, upload, retrieve])
        ).expect("valid rocket instance");

        db::delete("CD316D0E674E7B93663D0DE3F7C4D0B8751A4284".to_string(), &init_pool().get().unwrap()).unwrap();

        let mut response = client.post("/").body("-----BEGIN PGP PUBLIC KEY BLOCK-----

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
=KPfW
-----END PGP PUBLIC KEY BLOCK-----").dispatch();

        let mut response = client.get(URI::new(response.body_string().unwrap())).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Fingerprint: CD31 6D0E 674E 7B93 663D  0DE3 F7C4 D0B8 751A 4284

0: UID: Test User 2 <testuser2@testmail.com>, 1 self-signature(s), 0 certification(s)

0: Fingerprint: 9E67 875B C9D9 35DC BD4C  798E 2260 DC3A 7D1B F3EB, 1 self-signature(s), 0 certification(s)
".into()));

        db::delete("CD316D0E674E7B93663D0DE3F7C4D0B8751A4284".to_string(), &init_pool().get().unwrap()).unwrap();
    }

}
