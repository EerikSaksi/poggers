#![feature(test)]
mod build_schema;
mod server_side_json_builder;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[post("/graphql")]
async fn graphql(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(graphql))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
