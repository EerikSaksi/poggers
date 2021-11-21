#![feature(test)]
mod build_schema;
mod server_side_json_builder;

mod config {
    pub use ::config::ConfigError;
    use serde::Deserialize;
    #[derive(Deserialize)]
    pub struct Config {
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }
    impl Config {
        pub fn from_env() -> Result<Self, ConfigError> {
            let mut cfg = ::config::Config::new();
            cfg.merge(::config::Environment::new())?;
            cfg.try_into()
        }
    }
}
mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct GraphlQuery {
        pub data: String,
    }
}
mod handlers {
    use crate::{models::GraphlQuery, server_side_json_builder::ServerSidePoggers};
    use actix_web::{web, Error, HttpResponse};
    use deadpool_postgres::{Client, Pool};
    pub async fn poggers(
        query: web::Json<GraphlQuery>,
        poggers: web::Json<ServerSidePoggers>,
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let query_info: GraphlQuery = query.into_inner();
        println!("{}", query_info.data);
        Ok(HttpResponse::Ok().json("{\"bigFloppa\": \"cute\"}"))
    }
}

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use handlers::poggers;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();
    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(web::resource("/graphql").route(web::post().to(poggers)))
    })
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);
    server.await
}
