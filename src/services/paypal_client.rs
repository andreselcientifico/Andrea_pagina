use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct PayPalClient {
    pub client: Client,
    pub client_id: String,
    pub secret: String,
    pub base_url: String,
    pub access_token: tokio::sync::RwLock<String>,
}

impl PayPalClient {
    pub async fn new(client_id: String, secret: String, sandbox: bool) -> Self {
        let base_url = if sandbox {
            "https://api-m.sandbox.paypal.com".to_string()
        } else {
            "https://api-m.paypal.com".to_string()
        };

        let client = Client::new();

        let paypal = PayPalClient {
            client,
            client_id,
            secret,
            base_url,
            access_token: tokio::sync::RwLock::new(String::new()),
        };

        paypal.refresh_access_token().await.unwrap();

        paypal
    }

    /// Obtiene un nuevo token OAuth2
    pub async fn refresh_access_token(&self) -> Result<(), reqwest::Error> {
        let res = self.client.post(format!("{}/v1/oauth2/token", self.base_url))
            .basic_auth(&self.client_id, Some(&self.secret))
            .form(&[("grant_type", "client_credentials")])
            .send().await?;

        #[derive(Deserialize)]
        struct TokenRes {
            access_token: String,
        }

        let token: TokenRes = res.json().await?;
        *self.access_token.write().await = token.access_token;

        Ok(())
    }

    /// Headers con token
    pub async fn auth_header(&self) -> (String, String) {
        |this: &Self| async move {
            ("Authorization".into(), format!("Bearer {}", this.access_token.read().await))
        }
    }

    // -----------------------------------------------------------
    // 1. Crear PRODUCTO (para cursos)
    // -----------------------------------------------------------
    pub async fn create_product(&self, name: &str, description: &str)
        -> Result<String, reqwest::Error>
    {
        #[derive(Serialize)]
        struct ProductReq<'a> {
            name: &'a str,
            description: &'a str,
            r#type: &'a str,
            category: &'a str,
        }

        #[derive(Deserialize)]
        struct ProductRes {
            id: String,
        }

        let (h, v) = self.auth_header()(&self).await;

        let res = self.client.post(format!("{}/v1/catalogs/products", self.base_url))
            .header(h, v)
            .json(&ProductReq {
                name,
                description,
                r#type: "DIGITAL",
                category: "SOFTWARE",
            })
            .send().await?;

        let body: ProductRes = res.json().await?;
        Ok(body.id)
    }

    // -----------------------------------------------------------
    // 2. Crear ORDEN 
    // -----------------------------------------------------------
    pub async fn create_order(&self, amount: f64, description: &str)
        -> Result<String, reqwest::Error>
    {
        #[derive(Serialize)]
        struct Amount {
            currency_code: String,
            value: String,
        }

        #[derive(Serialize)]
        struct PurchaseUnit<'a> {
            amount: Amount,
            description: &'a str,
        }

        #[derive(Serialize)]
        struct OrderReq<'a> {
            intent: &'a str,
            purchase_units: Vec<PurchaseUnit<'a>>,
        }

        #[derive(Deserialize)]
        struct OrderRes {
            id: String,
        }

        let (h, v) = self.auth_header()(&self).await;

        let body = OrderReq {
            intent: "CAPTURE",
            purchase_units: vec![PurchaseUnit {
                amount: Amount {
                    currency_code: "USD".into(),
                    value: format!("{:.2}", amount),
                },
                description,
            }],
        };

        let res = self.client.post(format!("{}/v2/checkout/orders", self.base_url))
            .header(h, v)
            .json(&body)
            .send().await?;

        let body: OrderRes = res.json().await?;
        Ok(body.id)
    }

    // -----------------------------------------------------------
    // 3. Capturar ORDEN
    // -----------------------------------------------------------
    pub async fn capture_order(&self, order_id: &str)
        -> Result<String, reqwest::Error>
    {
        #[derive(Deserialize)]
        struct CaptureRes {
            id: String,
        }

        let (h, v) = self.auth_header()(&self).await;

        let res = self.client.post(format!(
            "{}/v2/checkout/orders/{}/capture",
            self.base_url, order_id
        ))
        .header(h, v)
        .send().await?;

        let body: CaptureRes = res.json().await?;
        Ok(body.id)
    }

    // -----------------------------------------------------------
    // 4. Crear SUSCRIPCIÓN (plan mensual)
    // -----------------------------------------------------------
    pub async fn create_subscription(&self, plan_id: &str)
        -> Result<String, reqwest::Error>
    {
        #[derive(Serialize)]
        struct SubReq<'a> {
            plan_id: &'a str,
        }

        #[derive(Deserialize)]
        struct SubRes {
            id: String,
        }

        let (h, v) = self.auth_header()(&self).await;

        let res = self.client.post(format!("{}/v1/billing/subscriptions", self.base_url))
            .header(h, v)
            .json(&SubReq { plan_id })
            .send().await?;

        let body: SubRes = res.json().await?;
        Ok(body.id)
    }
}
