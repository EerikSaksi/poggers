#![feature(test)]
mod build_schema;
mod server_side_json_builder;

//mod config {
//    pub use ::config::ConfigError;
//    use serde::Deserialize;
//    #[derive(Deserialize)]
//    pub struct Config {
//        pub server_addr: String,
//        pub pg: deadpool_postgres::Config,
//    }
//    impl Config {
//        pub fn from_env() -> Result<Self, ConfigError> {
//            let mut cfg = ::config::Config::new();
//            cfg.merge(::config::Environment::new())?;
//            cfg.try_into()
//        }
//    }
//}
//mod models {
//    use serde::{Deserialize, Serialize};
//
//    #[derive(Deserialize, Serialize, Debug)]
//    pub struct GraphlQuery {
//        pub data: String,
//    }
//}
//mod handlers {
//    use crate::{
//        models::GraphlQuery, server_side_json_builder::JsonBuilder,
//        server_side_json_builder::ServerSidePoggers,
//    };
//    use actix_web::{web, Error, HttpResponse};
//    use deadpool_postgres::{Client, Pool};
//    pub async fn poggers(
//        pool: web::Data<Pool>,
//        poggers: web::Data<ServerSidePoggers>,
//        query: web::Json<GraphlQuery>,
//    ) -> Result<HttpResponse, Error> {
//        let builder_context = poggers
//            .into_inner()
//            .build_root(&query.into_inner().data)
//            .unwrap();
//        let client: Client = pool.get().await.unwrap();
//        //let stmt = client.prepare(&builder_context.sql_query).await.unwrap();
//        let rows = client
//            .query_raw("select * from site_ser", [])
//            .await
//            .unwrap();
//        let res = JsonBuilder::new(builder_context).convert(rows);
//
//        //JsonBuilder::new().convert()
//        Ok(HttpResponse::Ok().json("{\"bigFloppa\": \"cute\"}"))
//    }
//}
//
//use actix_web::{web, App, HttpServer};
//use dotenv::dotenv;
//use handlers::poggers;
//use tokio_postgres::NoTls;
//
//#[actix_web::main]
//async fn main() -> std::io::Result<()> {
//    dotenv().ok();
//    let config = crate::config::Config::from_env().unwrap();
//    let pool = config.pg.create_pool(NoTls).unwrap();
//    let pogg = build_schema::create();
//    let server = HttpServer::new(move || {
//        App::new()
//            .data(pool.clone())
//            .data(pogg.clone())
//            .service(web::resource("/graphql").route(web::post().to(poggers)))
//    })
//    .bind(config.server_addr.clone())?
//    .run();
//    println!("Server running at http://{}/", config.server_addr);
//    server.await
//}
pub fn main() {
    
}
