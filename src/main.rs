mod build_schema;
mod server_side_json_builder;

#[derive(Debug, serde::Deserialize)]
struct Config {
    database_url: String,
    server_addr: String,
    pool_size: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new().separator("."))?;
        cfg.try_into()
    }
}

mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct GraphQLQuery {
        pub query: String,
    }
}
mod handlers {
    use crate::{
        models::GraphQLQuery, server_side_json_builder::JsonBuilder,
        server_side_json_builder::ServerSidePoggers,
    };
    use actix_web::{
        web::{Bytes, Data},
        HttpResponse,
    };
    use deadpool_postgres::Pool;
    fn format_err(e: String) -> HttpResponse {
        HttpResponse::Ok().json(format!("{{\"errors\": [\"message\": \"{}\"]}}", e))
    }
    pub async fn poggers(
        pool: Data<Pool>,
        poggers: Data<ServerSidePoggers>,
        query: Bytes,
    ) -> HttpResponse {
        let gql_query: GraphQLQuery =
            serde_json::from_str(std::str::from_utf8(&query).unwrap()).unwrap();
        match poggers.into_inner().build_root(&gql_query.query) {
            Ok(ctx) => {
                //acquire client and drop it as soon as we get the rows, prior to processing the data
                let rows = {
                    let client = match pool.get().await {
                        Ok(cli) => cli,
                        Err(e) => return format_err(e.to_string()),
                    };
                    match client.query(&*ctx.sql_query, &[]).await {
                        Ok(rows) => rows,
                        Err(e) => return format_err(e.to_string()),
                    }
                };
                let res = JsonBuilder::new(ctx).convert(rows);
                HttpResponse::Ok().json(res)
            }
            Err(e) => format_err(e),
        }
    }
}

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use handlers::poggers;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::str::FromStr;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let env_config = crate::Config::from_env().unwrap();
    println!("{}", &env_config.database_url);
    let config =
        deadpool_postgres::tokio_postgres::Config::from_str(&env_config.database_url).unwrap();
    
    // Create Ssl postgres connector without verification as required to connect to Heroku.
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    let manager = deadpool_postgres::Manager::new(config, MakeTlsConnector::new(builder.build()));
    let pool = deadpool_postgres::Pool::builder(manager).max_size(env_config.pool_size).build().unwrap();
    let pogg = build_schema::create(&pool).await;
    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(pogg.clone())
            .service(web::resource("/graphql").route(web::post().to(poggers)))
    })
    .bind(env_config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", env_config.server_addr);
    server.await
}
