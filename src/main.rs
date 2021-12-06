mod build_schema;
mod server_side_json_builder;

mod config {
    pub use ::config::ConfigError;
    use serde::Deserialize;
    #[derive(Deserialize, Debug)]
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
    use crate::{
        models::GraphlQuery, server_side_json_builder::JsonBuilder,
        server_side_json_builder::ServerSidePoggers,
    };
    use actix_web::{web, Error, HttpResponse};
    use deadpool_postgres::{Client, Pool};
    pub async fn poggers(
        pool: web::Data<Pool>,
        poggers: web::Data<ServerSidePoggers>,
        query: web::Json<GraphlQuery>,
    ) -> Result<HttpResponse, Error> {
        let query_str = &query.into_inner().data;
        println!("{}", query_str);
        let ctx = poggers.into_inner().build_root(query_str).unwrap();

        //acquire client and drop it as soon as we get the rows, prior to processing the data
        let rows = {
            let client: Client = pool.get().await.unwrap();
            client.query(&*ctx.sql_query, &[]).await.unwrap()
        };
        let res = JsonBuilder::new(ctx).convert(rows);
        Ok(HttpResponse::Ok().json(res))
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
    let pogg = build_schema::create(&pool).await;
    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(pogg.clone())
            .service(web::resource("/graphql").route(web::post().to(poggers)))
    })
    .workers(8)
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);
    server.await
}
