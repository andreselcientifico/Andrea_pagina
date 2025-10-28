use std::env;

// FIXME: construir config
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub paypal_api_mode: String,
    pub paypal_client_id: String,
    pub paypal_secret: String,
    pub port: u16,
}

// FIXME: usar init
impl Config {

    pub fn init() -> Config {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL no está seteada");
        let paypal_api_mode = env::var("PAYPAL_API_MODE").unwrap_or("https://api-m.sandbox.paypal.com".to_string());
        let paypal_client_id = env::var("PAYPAL_API_CLIENT_ID").expect("PAYPAL_API_CLIENT_ID no definido");
        let paypal_secret = env::var("PAYPAL_API_SECRET").expect("PAYPAL_API_SECRET no definido");

        Config {
            database_url,
            paypal_api_mode,
            paypal_client_id,
            paypal_secret,
            port: 8000,
        }
    }
}