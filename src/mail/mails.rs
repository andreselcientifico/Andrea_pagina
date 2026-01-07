use super::sendmail::send_email;

pub async fn send_verification_email(
    to_email: &str,
    username: &str,
    token: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Verificación de Correo Electrónico";
    let base_url = "https://localhost:8000/api/auth/verify";
    let verification_link = create_verification_link(base_url, token);

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
        username,
        verification_link
    );

    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{verification_link}}".to_string(), verification_link)
    ];

    send_email(to_email, subject, &body_html, &placeholders).await
}

fn create_verification_link(base_url: &str, token: &str) -> String {
    format!("{}?token={}", base_url, token)
}

#[allow(dead_code)]
pub async fn send_welcome_email(
    to_email: &str,
    username: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "¡Bienvenido a Vallenato Academy! 🎉";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string())
    ];

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

#[allow(dead_code)]
pub async fn send_forgot_password_email(
    to_email: &str,
    reset_link: &str,
    username: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Restablecer tu Contraseña 🔒";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{reset_link}}".to_string(), reset_link.to_string())
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
        username,
        reset_link
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}