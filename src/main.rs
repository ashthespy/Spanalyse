extern crate futures;
extern crate librespot;
extern crate serde_json;
extern crate tokio_core;
#[macro_use]
extern crate log;
extern crate env_logger;

use env_logger::Builder;
use futures::Future;
use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::SpotifyId;
use serde_json::Value;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use tokio_core::reactor::Core;

fn main() {
    let mut builder = Builder::new();
    builder.parse("librespot=info,spanalyse=info");
    builder.init();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let session_config = SessionConfig::default();

    let args: Vec<_> = env::args().collect();
    // Std test uri
    let mut uri = String::from("hm://radio-apollo/v3/stations/spotify:track:2J56abZOk2tv0GyePJnAYN");

    if args.len() > 3 {
        uri = args[3].to_owned();
    }
    if args.len() > 4 {
        let spotify_id = SpotifyId::from_base62(&args[4]).unwrap();
        info!(
            "SpotifyId {:?}\nbase_16 {:?}\nbase_62 {:?}",
            spotify_id,
            spotify_id.to_base16(),
            spotify_id.to_base62()
        );
    }

    let username = args[1].to_owned();
    let password = args[2].to_owned();

    let credentials = Credentials::with_password(username, password);

    info!("Connecting ..");
    let session = core
        .run(Session::connect(session_config, credentials, None, handle))
        .unwrap();

    info!("Attempting to get uri {:?}", uri);
    core.run(session.mercury().get(uri).and_then(move |response| {
        debug!("{:?}", response);
        let bytes = response.payload.first().unwrap().clone();
        info!("Bytes {:?}", bytes.len());
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("bytes.txt")
            .unwrap();
        file.write_all(&bytes).unwrap();
        drop(file);
        let data = String::from_utf8(response.payload.first().unwrap().clone()).unwrap();
        let value: Value = serde_json::from_str(&data).unwrap();
        info!("Response: {:?}", value);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("output.json")
            .unwrap();
        serde_json::to_writer(&mut file, &value).unwrap();

        Ok(())
    })).expect("MercuryError:: ");
}
