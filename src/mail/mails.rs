use super::sendmail::send_email;
use crate::{AppState, utils::token::base_url};

pub async fn send_verification_email(
    to_email: &str,
    app_state: &AppState,
    username: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Verificación de Correo Electrónico";
    let base_url = format!("{}verificar-email", base_url(&app_state.env.host));
    let verification_link = create_verification_link(&base_url, token);

    let body_html = format!(
        r#"
        <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }}
                    .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; }}
                    .button {{ display: inline-block; background: #4CAF50; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; }}
                    .footer {{ background: #f4f4f4; padding: 10px; text-align: center; font-size: 12px; color: #666; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>¡Hola, {}! 👋</h1>
                </div>
                <div class="content">
                    <p>Gracias por registrarte en nuestra plataforma de aprendizaje.</p>
                    <p>Para completar tu registro, por favor verifica tu correo electrónico haciendo clic en el siguiente enlace:</p>
                    <p style="text-align: center; margin: 30px 0;">
                        <a href="{}" class="button">Verificar Correo Electrónico</a>
                    </p>
                    <p>Si no solicitaste esta acción, puedes ignorar este correo de manera segura.</p>
                </div>
                <div class="footer">
                    <p>Equipo de Vallenato Academy</p>
                </div>
            </body>
        </html>
        "#,
        username, verification_link
    );

    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{verification_link}}".to_string(), verification_link),
    ];

    send_email(to_email, subject, &body_html, &placeholders).await
}

fn create_verification_link(base_url: &String, token: &str) -> String {
    format!("{}?token={}", base_url, token)
}

pub async fn send_welcome_email(
    to_email: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "¡Bienvenido a Vallenato Academy! 🎉";
    let placeholders = vec![("{{username}}".to_string(), username.to_string())];

    let body_html = format!(
        r#"
        <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }}
                    .header {{ background: linear-gradient(135deg, #764ba2 0%, #667eea 100%); color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; }}
                    .footer {{ background: #f4f4f4; padding: 10px; text-align: center; font-size: 12px; color: #666; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>¡Bienvenido, {}! 🎉</h1>
                </div>
                <div class="content">
                    <p>¡Felicidades! Tu cuenta ha sido verificada exitosamente.</p>
                    <p>Ahora puedes acceder a todos nuestros cursos de vallenato y comenzar tu viaje de aprendizaje.</p>
                    <p>¡Esperamos que disfrutes tu experiencia de aprendizaje!</p>
                </div>
                <div class="footer">
                    <p>Equipo de Vallenato Academy</p>
                </div>
            </body>
        </html>
        "#,
        username
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}

pub async fn send_forgot_password_email(
    to_email: &str,
    reset_link: &str,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Restablecer tu Contraseña 🔒";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{reset_link}}".to_string(), reset_link.to_string()),
    ];

    let body_html = format!(
        r#"
        <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }}
                    .header {{ background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%); color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; }}
                    .button {{ display: inline-block; background: #FF5722; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; }}
                    .footer {{ background: #f4f4f4; padding: 10px; text-align: center; font-size: 12px; color: #666; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>Restablecer Contraseña 🔒</h1>
                </div>
                <div class="content">
                    <p>Hola, {}.</p>
                    <p>Hemos recibido una solicitud para restablecer tu contraseña. Si fuiste tú quien lo solicitó, haz clic en el siguiente enlace para crear una nueva contraseña:</p>
                    <p style="text-align: center; margin: 30px 0;">
                        <a href="{}" class="button">Restablecer Contraseña</a>
                    </p>
                    <p><strong>Este enlace expirará en 1 hora por seguridad.</strong></p>
                    <p>Si no solicitaste este cambio, puedes ignorar este correo. Tu contraseña actual seguirá siendo válida.</p>
                </div>
                <div class="footer">
                    <p>Equipo de Vallenato Academy</p>
                </div>
            </body>
        </html>
        "#,
        username, reset_link
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}

/// Notifies a user about new content (course/lesson) being added
pub async fn send_new_content_email(
    to_email: &str,
    username: &str,
    content_type: &str, // "curso" or "lección"
    content_title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = format!("🎵 ¡Nuevo {} disponible!", content_type);
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{content_type}}".to_string(), content_type.to_string()),
        ("{{content_title}}".to_string(), content_title.to_string()),
    ];

    let body_html = format!(
        r#"
        <html>
            <head>
                <style>
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

    send_email(to_email, &subject, &body_html, &placeholders).await
}

/// Sends a custom administrative email to users
pub async fn send_admin_bulk_email(
    to_email: &str,
    username: &str,
    subject: &str,
    html_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let placeholders = vec![("{{username}}".to_string(), username.to_string())];

    let body_html = format!(
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
                    <h1>🎹 Vallenato Academy</h1>
                </div>
                <div class="content">
                    <p>¡Hola, {}! 👋</p>
                    {}
                </div>
                <div class="footer">
                    <p>Equipo de Vallenato Academy</p>
                    <p style="font-size: 10px; color: #999;">Si no deseas recibir estas notificaciones, puedes desactivarlas en la configuración de tu perfil.</p>
                </div>
            </body>
        </html>
        "#,
        username, html_content
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}

pub async fn send_contact_email(
    name: &str,
    from_email: &str,
    subject_input: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| {
        std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "admin@vallenatofemenino.com".to_string())
    });

    let subject = format!("📬 Nuevo mensaje de contacto: {}", subject_input);

    let body_html = format!(
        r#"
        <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; color: #333; }}
                    .header {{ background: #f8f9fa; padding: 20px; border-bottom: 2px solid #eee; }}
                    .content {{ padding: 20px; }}
                    .footer {{ background: #f8f9fa; padding: 10px; text-align: center; font-size: 12px; color: #999; }}
                    .field {{ margin-bottom: 15px; }}
                    .label {{ font-weight: bold; color: #666; }}
                    .message-box {{ background: #f5f5f5; padding: 15px; border-radius: 5px; margin-top: 10px; border: 1px solid #ddd; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h2>Nuevo mensaje de contacto 📬</h2>
                </div>
                <div class="content">
                    <div class="field">
                        <span class="label">Nombre:</span> {}
                    </div>
                    <div class="field">
                        <span class="label">Email de contacto:</span> {}
                    </div>
                    <div class="field">
                        <span class="label">Asunto:</span> {}
                    </div>
                    <div class="field">
                        <span class="label">Mensaje:</span>
                        <div class="message-box">
                            {}
                        </div>
                    </div>
                </div>
                <div class="footer">
                    <p>Este mensaje fue enviado desde el formulario de contacto de Vallenato Academy.</p>
                </div>
            </body>
        </html>
        "#,
        name, from_email, subject_input, message
    );

    let placeholders = vec![
        ("{{name}}".to_string(), name.to_string()),
        ("{{email}}".to_string(), from_email.to_string()),
        ("{{subject}}".to_string(), subject_input.to_string()),
        ("{{message}}".to_string(), message.to_string()),
    ];

    send_email(&admin_email, &subject, &body_html, &placeholders).await
}
