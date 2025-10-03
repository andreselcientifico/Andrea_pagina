// FIXME: construir config
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub port: u16,
}

// FIXME: usar init
impl Config {

    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL no está seteada");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET no está seteada");
        let jwt_expiration = std::env::var("JWT_EXPIRATION").expect("JWT_EXPIRATION no está seteada").parse::<i64>().expect("JWT_EXPIRATION debe ser un número");

        Config {
            database_url,
            jwt_secret,
            jwt_expiration,
            port: 8000,
        }
    }
}