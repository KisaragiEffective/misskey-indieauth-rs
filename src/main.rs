#![allow(warnings)] // うるさいのでしばしの間消えてもらう
use actix_web::{App, get, head, HttpResponseBuilder, HttpServer, Responder};
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use clap::Parser;
use log::info;
use serde::Serialize;
use url::Url;

// TODO:
//   https://{MISSKEY}/oauth/authorize?response_type=code&client_id={CLIENT_HOST}/profile&redirect_url={CLIENT_HOST}/authorize
//   yields "invalid_request" error.
//   What was went wrong?
fn profile0(data: Data<AppState>) -> impl Responder {
    let exposed_host = &data.get_ref().host_and_scheme;
    HttpResponseBuilder::new(StatusCode::NO_CONTENT)
        .insert_header(("Link", format!("<{exposed_host}/metadata>; rel=\"indieauth-metadata\",\
        <{exposed_host}/authorize/>; rel=\"redirect_uri\",\
        <{exposed_host}/authorize>; rel=\"redirect_uri\"")))
        .finish()
}

#[get("/profile")]
async fn profile(data: Data<AppState>) -> impl Responder {
    profile0(data)
}

#[head("/profile")]
async fn profile_head(data: Data<AppState>) -> impl Responder {
    profile0(data)
}

#[derive(Serialize, Debug)]
struct IndieAuthServerMetadata {
    issuer: Url,
    authorization_endpoint: Url,
    token_endpoint: Url,
    introspection_endpoint: Url,
    code_challenge_methods_supported: Vec<String>
}

#[get("/metadata")]
async fn metadata(data: Data<AppState>) -> impl Responder {
    let exposed_host = &data.get_ref().host_and_scheme;
    let res = IndieAuthServerMetadata {
        issuer: Url::parse(&exposed_host).expect("failed to parse URL"),
        authorization_endpoint: Url::parse(&format!("{exposed_host}/authorize/")).expect("failed to parse authorize endpoint as a URL"),
        token_endpoint: Url::parse(&format!("{exposed_host}/token/")).expect("failed to parse token exchange endpoint as a URL"),
        introspection_endpoint: Url::parse(&format!("{exposed_host}/introspection/")).expect("failed to parse introspection endpoint as a URL"),
        code_challenge_methods_supported: vec!["S256".to_string()]
    };

    info!("{}", serde_json::to_string(&res).unwrap());

    HttpResponseBuilder::new(StatusCode::OK)
        .json(res)
}

#[get("/authorize")]
async fn authorize(data: Data<AppState>) -> impl Responder {
    unimplemented!();
    ""
}

#[get("/token")]
async fn exchange_token(data: Data<AppState>) -> impl Responder {
    unimplemented!();
    ""
}

#[get("/introspection")]
async fn introspection(data: Data<AppState>) -> impl Responder {
    panic!();
    ""
}

struct AppState {
    host_and_scheme: String
}

#[derive(Parser)]
struct Args {
    host: String,
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();
    let host = args.host;

    fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "[{} {}] {}",
            record.level(),
            record.target(),
            message,
        ))
    }).chain(
        fern::Dispatch::new()
            .level(log::LevelFilter::Debug)
            // by default only accept warn messages
            .chain(std::io::stdout())
    ).apply().expect("failed to initialize logger");

    HttpServer::new(move || App::new()
        .app_data(actix_web::web::Data::new(AppState {
            host_and_scheme: format!("https://{host}")
        }))
        .service((profile, profile_head, metadata, authorize, exchange_token, introspection))
        .wrap(Logger::new("%a (%{CF-Connecting-IP}i) %r %s %U")))
        .bind(("127.0.0.1", 62192))
        .expect("failed to initialize HTTP server").run().await.expect("error has occurred in HTTP server");
}
