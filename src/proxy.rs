#![allow(dead_code)]
#![allow(unused_imports)]

use super::key::*;
use super::encrypt::*;
use actix_multipart::{Multipart};
use actix_web::client::Client;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use clap::{value_t, Arg};
use futures::Future;
use std::net::ToSocketAddrs;
use url::Url;

fn forward(
    _req: HttpRequest,
    mut payload: web::Payload,
    client: web::Data<Client>,
) -> impl Future<Item = HttpResponse, Error = Error> {


    let url = "https://storage.gra5.cloud.ovh.net/***";

    let key = build_key();
    let encoder = Encoder::new(key, &mut payload);

    client.put(url)
        .header("User-Agent", "Actix-web")
        .send_stream(encoder)                             // <- Send http request
        .map_err(|e| {
            println!("==== erreur1 ====");
            println!("{:?}", e);
            Error::from(e)
        })
    .map(|res| {
        let mut client_resp = HttpResponse::build(res.status());
        for (header_name, header_value) in
            res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }
        client_resp.streaming(res)
    })
}

pub fn main() -> std::io::Result<()> {
    let matches = clap::App::new("HTTP Proxy")
        .arg(
            Arg::with_name("listen_addr")
            .takes_value(true)
            .value_name("LISTEN ADDR")
            .index(1)
            .required(true),
            )
        .arg(
            Arg::with_name("listen_port")
            .takes_value(true)
            .value_name("LISTEN PORT")
            .index(2)
            .required(true),
            )
        .get_matches();

    let listen_addr = matches.value_of("listen_addr").unwrap();
    let listen_port = value_t!(matches, "listen_port", u16).unwrap_or_else(|e| e.exit());

    HttpServer::new(move || {
        App::new()
            .data(actix_web::client::Client::new())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to_async(forward))
    })
    .bind((listen_addr, listen_port))?
        .system_exit()
        .run()
}
