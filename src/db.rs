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
    
    let host = env::var("DATABASE_HOST")
        .expect("Could not load DATABASE_HOST from environment. Contact Metalface if you're seeing this and you're not him.");
    let dbname = env::var("DATABASE_NAME")
        .expect("Could not load DATABASE_NAME from environment. Contact Metalface if you're seeing this and you're not him.");
    let user = env::var("DATABASE_USERNAME")
        .expect("Could not load DATABASE_USERNAME from environment. Contact Metalface if you're seeing this and you're not him.");
    let password = env::var("DATABASE_PASSWORD")
        .expect("Could not load DATABASE_PASSWORD from environment. Contact Metalface if you're seeing this and you're not him.");


    let params = DatabaseParameters { host, dbname, user, password };
    return params;
}