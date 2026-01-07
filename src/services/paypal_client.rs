use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PayPalClient {
    pub client: Client,
    pub client_id: String,
    pub secret: String,
    pub base_url: String,
    pub access_token: Arc<tokio::sync::RwLock<String>>,
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
            access_token: Arc::new(tokio::sync::RwLock::new(String::new())),
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
        ("Authorization".into(), format!("Bearer {}", self.access_token.read().await))
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

        let (h, v) = self.auth_header().await;

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

        let (h, v) = self.auth_header().await;

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

        let (h, v) = self.auth_header().await;

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

        let (h, v) = self.auth_header().await;

        let res = self.client.post(format!("{}/v1/billing/subscriptions", self.base_url))
            .header(h, v)
            .json(&SubReq { plan_id })
            .send().await?;

        let body: SubRes = res.json().await?;
        Ok(body.id)
    }
    pub async fn create_plan(&self, product_id: &str, name: &str, description: &str, price: f64, interval: &str, interval_count: i32)
        -> Result<String, reqwest::Error>
    {
        #[derive(Serialize)]
        struct PricingScheme {
            fixed_price: Amount,
        }

        #[derive(Serialize)]
        struct Frequency {
            interval_unit: String,
            interval_count: i32,
        }

        #[derive(Serialize)]
        struct BillingCycle {
            frequency: Frequency,
            tenure_type: String,
            sequence: i32,
            total_cycles: i32,
            pricing_scheme: PricingScheme,
        }

        #[derive(Serialize)]
        struct Amount {
            currency_code: String,
            value: String,
        }

        #[derive(Serialize)]
        struct PlanReq<'a> {
            product_id: &'a str,
            name: &'a str,
            description: &'a str,
            status: &'a str,
            billing_cycles: Vec<BillingCycle>,
            payment_preferences: PaymentPreferences,
        }

        #[derive(Serialize)]
        struct PaymentPreferences {
            auto_bill_outstanding: bool,
            setup_fee_failure_action: String,
            payment_failure_threshold: i32,
        }

        #[derive(Deserialize)]
        struct PlanRes {
            id: String,
        }

        let (h, v) = self.auth_header().await;

        let body = PlanReq {
            product_id,
            name,
            description,
            status: "ACTIVE",
            billing_cycles: vec![BillingCycle {
                frequency: Frequency {
                    interval_unit: interval.to_uppercase(),
                    interval_count,
                },
                tenure_type: "REGULAR".to_string(),
                sequence: 1,
                total_cycles: 0, // 0 significa indefinido
                pricing_scheme: PricingScheme {
                    fixed_price: Amount {
                        currency_code: "USD".to_string(),
                        value: format!("{:.2}", price),
                    },
                },
            }],
            payment_preferences: PaymentPreferences {
                auto_bill_outstanding: true,
                setup_fee_failure_action: "CANCEL".to_string(),
                payment_failure_threshold: 3,
            },
        };

        let res = self.client.post(format!("{}/v1/billing/plans", self.base_url))
            .header(h, v)
            .json(&body)
            .send().await?;

        let body: PlanRes = res.json().await?;
        Ok(body.id)
    }

    // -----------------------------------------------------------
    // 6. Eliminar PRODUCTO
    // -----------------------------------------------------------
    pub async fn delete_product(&self, product_id: &str)
        -> Result<(), reqwest::Error>
    {
        let (h, v) = self.auth_header().await;

        let _res = self.client.delete(format!("{}/v1/catalogs/products/{}", self.base_url, product_id))
            .header(h, v)
            .send().await?;

        Ok(())
    }

    // -----------------------------------------------------------
    // 7. Eliminar PLAN
    // -----------------------------------------------------------
    pub async fn delete_plan(&self, plan_id: &str)
        -> Result<(), reqwest::Error>
    {
        let (h, v) = self.auth_header().await;

        let _res = self.client.delete(format!("{}/v1/billing/plans/{}", self.base_url, plan_id))
            .header(h, v)
            .send().await?;

        Ok(())
    }

    // -----------------------------------------------------------
    // 8. Cancelar SUSCRIPCIÓN
    // -----------------------------------------------------------
    pub async fn cancel_subscription(&self, subscription_id: &str)
        -> Result<(), reqwest::Error>
    {
        let (h, v) = self.auth_header().await;

        let _res = self.client.post(format!("{}/v1/billing/subscriptions/{}/cancel", self.base_url, subscription_id))
            .header(h, v)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "reason": "User requested cancellation"
            }))
            .send().await?;

        Ok(())
    }
}
