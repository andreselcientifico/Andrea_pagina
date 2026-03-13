use lettre::{
    Message, SmtpTransport, Transport,
    message::{SinglePart, header},
    transport::smtp::authentication::Credentials,
};
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;
use std::{env, string};

// pub async fn send_email_gmail(
//     to_email: &str,
//     subject: &str,
//     body_template: &String,
//     placeholders: &[(String, String)],
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Cargar variables de entorno
//     let smtp_username = match env::var("SMTP_USERNAME") {
//         Ok(val) => val,
//         Err(_) => {
//             return Err("SMTP_USERNAME missing".into());
//         }
//     };

//     let smtp_password = match env::var("SMTP_PASSWORD") {
//         Ok(val) => val,
//         Err(_) => {
//             return Err("SMTP_PASSWORD missing".into());
//         }
//     };

//     let smtp_server = match env::var("SMTP_SERVER") {
//         Ok(val) => val,
//         Err(_) => {
//             return Err("SMTP_SERVER missing".into());
//         }
//     };

//     let smtp_port: u16 = match env::var("SMTP_PORT") {
//         Ok(val) => val.parse().unwrap_or(587),
//         Err(_) => 587,
//     };

//     let mut body = body_template.to_string();
//     for (key, value) in placeholders {
//         body = body.replace(key, value);
//     }

//     let email = Message::builder()
//         .from(smtp_username.parse()?)
//         .to(to_email.parse()?)
//         .subject(subject)
//         .header(header::ContentType::TEXT_HTML)
//         .singlepart(
//             SinglePart::builder()
//                 .header(header::ContentType::TEXT_HTML)
//                 .body(body),
//         )?;

//     let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());
//     let mailer = SmtpTransport::starttls_relay(&smtp_server)?
//         .credentials(creds)
//         .port(smtp_port)
//         .build();

//     match mailer.send(&email) {
//         Ok(_) => (),
//         Err(e) => {
//             return Err(format!("Could not send email: {:?}", e).into());
//         }
//     }

//     Ok(())
// }

pub async fn send_email(
    to_email: &str,
    subject: &str,
    body_template: &String,
    placeholders: &[(String, String)],
    from: impl Into<String>,
    reply_to: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Reemplazar placeholders (igual que antes)
    let mut body = body_template.to_string();
    for (key, value) in placeholders {
        body = body.replace(key, value);
    }

    // 2. Crear cliente Resend
    let api_key = env::var("RESEND_API_KEY").map_err(|e| {
        log::error!("Error Resend: {}", e);
        e.to_string()
    })?;

    let resend = Resend::new(&api_key);

    // 3. Enviar con la librería oficial
    let email: CreateEmailBaseOptions = CreateEmailBaseOptions::new(from, vec![to_email], subject).with_html(&body).with_reply(reply_to.unwrap()); 

    resend.emails.send(email).await.map_err(|e| {
        log::error!("Error Resend: {}", e);
        e.to_string()
    })?;

    Ok(())
}

pub async fn send_bulk_notification(
    users: Vec<(&str, &str)>, // Vec de (email, nombre)
    subject: &str,
    html_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("RESEND_API_KEY").map_err(|e| {
        log::error!("Error Resend: {}", e);
        e.to_string()
    })?;
    let resend = Resend::new(&api_key);

    // Resend batch API limita a 100 emails por llamada
    const BATCH_SIZE: usize = 100;

    for chunk in users.chunks(BATCH_SIZE) {
        let emails: Vec<CreateEmailBaseOptions> = chunk
            .iter()
            .map(|(email, name)| {
                let body = format!(
                    r#"
                    <html>
                        <head>
                            <style>
                                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }}
                                .header {{ background: linear-gradient(135deg, #764ba2 0%, #667eea 100%); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }}
                                .content {{ padding: 30px; background: #ffffff; line-height: 1.6; }}
                                .footer {{ background: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #666; border-radius: 0 0 8px 8px; }}
                            </style>
                        </head>
                        <body>
                            <div class="header">
                                <h1>🎹 Vallenato Femenino</h1>
                            </div>
                            <div class="content">
                                <p>¡Hola, {}! 👋</p>
                                {}
                            </div>
                            <div class="footer">
                                <p>Equipo de Vallenato Femenino</p>
                                <p style="font-size: 10px; color: #999;">Si no deseas recibir estas notificaciones, puedes desactivarlas en la configuración de tu perfil.</p>
                            </div>
                        </body>
                    </html>
                    "#,
                    name, html_content
                );

                CreateEmailBaseOptions::new(
                    "Vallenato Femenino <contacto@vallenatofemenino.com>",
                    vec![*email],
                    subject,
                )
                .with_html(&body)
            })
            .collect();

        resend.batch.send(emails).await.map_err(|e| {
            log::error!("Error Resend batch: {}", e);
            e.to_string()
        })?;

        // Pequeña pausa entre batches para no sobrecargar
        if chunk.len() == BATCH_SIZE {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    Ok(())
}

pub async fn send_bulk_new_content_emails(
    users: Vec<(String, String)>, // Vector de (email, nombre)
    content_type: &str,
    content_title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("RESEND_API_KEY").map_err(|e| e.to_string())?;
    let resend = Resend::new(&api_key);
    let subject = format!("🎵 ¡Nuevo {} disponible!", content_type);
    let from = "Vallenato Femenino <admin@vallenatofemenino.com>";

    let mut batch_emails = Vec::new();

    // 1. Armamos las opciones para cada usuario
    for (email, username) in &users {
        let body_html = format!(
            r#"
            <html>
                <head>
                    <style>
                        /* Tus estilos originales se mantienen aquí */
                        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }}
                        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }}
                        .content {{ padding: 30px; background: #ffffff; }}
                        .highlight {{ background: #f5f3ff; border-left: 4px solid #764ba2; padding: 15px; margin: 20px 0; border-radius: 4px; }}
                        .button {{ display: inline-block; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 14px 28px; text-decoration: none; border-radius: 8px; font-weight: bold; }}
                        .footer {{ background: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #666; border-radius: 0 0 8px 8px; }}
                    </style>
                </head>
                <body>
                    <div class="header">
                        <h1>🎵 ¡Nuevo Contenido!</h1>
                    </div>
                    <div class="content">
                        <p>¡Hola, {}! 👋</p>
                        <p>Tenemos excelentes noticias para ti. Acabamos de agregar nuevo contenido a nuestra plataforma:</p>
                        <div class="highlight">
                            <strong>Nuevo {}:</strong> {}
                        </div>
                        <p>No te pierdas esta oportunidad de seguir mejorando tus habilidades con el acordeón.</p>
                        <p style="text-align: center; margin: 30px 0;">
                            <a href="https://academiadevallenato.com/mis-cursos" class="button">Ver Ahora</a>
                        </p>
                    </div>
                    <div class="footer">
                        <p>Equipo de Vallenato Academy</p>
                        <p style="font-size: 10px; color: #999;">Si no deseas recibir estas notificaciones, puedes desactivarlas en la configuración de tu perfil.</p>
                    </div>
                </body>
            </html>
            "#,
            username, content_type, content_title
        );

        let options = CreateEmailBaseOptions::new(from, vec![email.as_str()], &subject)
            .with_html(&body_html);

        batch_emails.push(options);
    }

    // 2. La API de Batch de Resend permite un máximo de 100 correos por petición
    for chunk in batch_emails.chunks(100) {
        if let Err(e) = resend.batch.send(chunk.to_vec()).await {
            log::error!("Error enviando batch de Resend: {}", e);
        }
    }

    Ok(())
}