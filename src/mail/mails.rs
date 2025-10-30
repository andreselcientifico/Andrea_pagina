use super::sendmail::send_email;

pub async fn send_verification_email(
    to_email: &str,
    username: &str,
    token: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Email Verification";
    let base_url = "https://localhost:8000/api/auth/verify";
    let verification_link = create_verification_link(base_url, token);
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{verification_link}}".to_string(), verification_link.clone())
    ];
    // Generamos el body HTML directamente
    let body_html = format!(
        r#"
        <html>
            <body>
                <h2>Hola, {username}!</h2>
                <p>Gracias por registrarte. Por favor verifica tu correo haciendo click en el siguiente enlace:</p>
                <a href="{verification_link}">Verificar correo</a>
                <p>Si no solicitaste esta acción, ignora este correo.</p>
            </body>
        </html>
        "#,
        username = username,
        verification_link = verification_link
    );

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
    let subject = "Welcome to Application";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string())
    ];

    let body_html = format!(
        r#"
        <html>
            <body>
                <h2>Hola, {username}!</h2>
                <p>Bienvenido a nuestra aplicación.</p>
            </body>
        </html>
        "#,
        username = username
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}

#[allow(dead_code)]
pub async fn send_forgot_password_email(
    to_email: &str,
    reset_link: &str,
    username: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = "Rest your Password";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{rest_link}}".to_string(), reset_link.to_string())
    ];

    let body_html = format!(
        r#"
        <html>
            <body>
                <h2>Hola, {username}!</h2>
                <p>Para restablecer tu contraseña, haz click en el siguiente enlace:</p>
                <a href="{reset_link}">Restablecer contraseña</a>
            </body>
        </html>
        "#,
        username = username,
        reset_link = reset_link
    );

    send_email(to_email, subject, &body_html, &placeholders).await
}