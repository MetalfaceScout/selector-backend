use std::env;

use dotenvy::dotenv;
use postgres::Client;
use postgres_native_tls::MakeTlsConnector;
use native_tls::TlsConnector;

use crate::DatabaseParameters;


pub fn create_db_client(params : DatabaseParameters) -> postgres::Client {

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build().unwrap();
    let tls = MakeTlsConnector::new(connector);

    let db_params = format!("host={} port=5432 dbname={} user={} password={} sslmode=require", params.host, params.dbname, params.user, params.password);

    let client = Client::connect(
        &db_params, 
        tls
    );

    return client.expect("Unable to connect to lfstats database");
}



pub fn set_database_params() -> DatabaseParameters {

    dotenv().expect("Failed to load .env file");
    
    let host = env::var("DATABASE_HOST").unwrap();
    let dbname = env::var("DATABASE_NAME").unwrap();
    let user = env::var("DATABASE_USERNAME").unwrap();
    let password = env::var("DATABASE_PASSWORD").unwrap();

    let params = DatabaseParameters { host, dbname, user, password };
    return params;
}