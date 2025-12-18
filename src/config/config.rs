use std::{env, fs};
use jsonwebtoken::{EncodingKey, DecodingKey};

// FIXME: construir config
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub paypal_api_mode: String,
    pub jwt_maxage: i64,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub paypal_client_id: String,
    pub paypal_secret: String,
    pub host: String,
    pub port: u16,
    pub paypal_webhook_id: String,
}

// FIXME: usar init
impl Config {

    pub fn init() -> Config {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL no está seteada");
        let paypal_api_mode = env::var("PAYPAL_API_MODE").unwrap_or("https://api-m.sandbox.paypal.com".to_string());
        let jwt_maxage = env::var("JWT_MAXAGE").unwrap_or("3600".to_string()).parse().unwrap_or(3600);
        let private_key = fs::read("private.pem").expect("No se pudo leer private.pem");
        let public_key = fs::read("public.pem").expect("No se pudo leer public.pem");
        let encoding_key = EncodingKey::from_rsa_pem(&private_key).expect("Error al construir Encodingkey");
        let decoding_key = DecodingKey::from_rsa_pem(&public_key).expect("Error al construir DecodingKey");
        let paypal_client_id = env::var("PAYPAL_API_CLIENT_ID").expect("PAYPAL_API_CLIENT_ID no definido");
        let paypal_secret = env::var("PAYPAL_API_SECRET").expect("PAYPAL_API_SECRET no definido");
        let paypal_webhook_id = env::var("PAYPAL_WEBHOOK_ID").expect("PAYPAL_WEBHOOK_ID no definido");
        let host = env::var("HOST").unwrap_or("localhost".to_string());

        Config {
            database_url,
            paypal_api_mode,
            jwt_maxage,
            private_key,
            public_key,
            encoding_key,
            decoding_key,
            paypal_client_id,
            paypal_secret,
            host,
            port: 8000,
            paypal_webhook_id,
        }
    }
}