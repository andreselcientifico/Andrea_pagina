use std::{env};
use lettre::{
    message::{header, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport,
    Transport,
};

pub async fn send_email(
    to_email: &str,
    subject: &str,
    body_template: &String,
    placeholders: &[(String, String)]
) -> Result<(), Box<dyn std::error::Error>> {
     // Cargar variables de entorno
    let smtp_username = match env::var("SMTP_USERNAME") {
        Ok(val) => val,
        Err(_) => {
            return Err("SMTP_USERNAME missing".into());
        }
    };

    let smtp_password = match env::var("SMTP_PASSWORD") {
        Ok(val) => val,
        Err(_) => {
            return Err("SMTP_PASSWORD missing".into());
        }
    };

    let smtp_server = match env::var("SMTP_SERVER") {
        Ok(val) => val,
        Err(_) => {
            return Err("SMTP_SERVER missing".into());
        }
    };

    let smtp_port: u16 = match env::var("SMTP_PORT") {
        Ok(val) => val.parse().unwrap_or(587),
        Err(_) => {
            587
        }
    };

    let mut body = body_template.to_string();
    for (key, value) in placeholders {
        body = body.replace(key, value);
    }

    let email = Message::builder()
        .from(smtp_username.parse()?)
        .to(to_email.parse()?)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .singlepart(SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(body)
        )?;

    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());
    let mailer = SmtpTransport::starttls_relay(&smtp_server)?
        .credentials(creds)
        .port(smtp_port)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("✅ Email enviado correctamente!"),
        Err(e) => println!("❌ Falló el envío de email: {:?}", e),
    }

    Ok(())
}