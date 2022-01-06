mod build_schema;
mod server_side_json_builder;

#[derive(Debug, serde::Deserialize)]
struct Config {
    pg: deadpool_postgres::Config,
    server_addr: String,
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

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::str::FromStr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut config = tokio_postgres::Config::from_str("postgres://tcnsakookamdap:09cc3c55a51ded8af63d9d97de356823add4b8991ad92b2349d5e58e723e4685@ec2-54-158-232-223.compute-1.amazonaws.com:5432/d287k6sulohu6l?sslmode=require").unwrap();
    // Create Ssl postgres connector without verification as required to connect to Heroku.
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    let manager = deadpool_postgres::Manager::new(config, MakeTlsConnector::new(builder.build()));
    let pool = deadpool_postgres::Pool::builder(manager).build().unwrap();
    let client = pool.get().await.unwrap();
    let res = client.query("select count(*) from tracks", &[]).await.unwrap();
    let res: i64 = res.get(0).unwrap().get(0);
    println!("{:?}", res);
    Ok(())
}
