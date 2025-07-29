use lettre::message::{header, MultiPart, SinglePart, Attachment};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::path::Path;
use crate::models::EmailConfig;
use anyhow::{Result, anyhow};
use log::{info, error};

pub struct EmailService {
    config: Option<EmailConfig>,
}

impl EmailService {
    pub fn new() -> Self {
        EmailService { config: None }
    }
    
    pub fn with_config(config: EmailConfig) -> Self {
        EmailService {
            config: Some(config),
        }
    }
    
    pub fn set_config(&mut self, config: EmailConfig) {
        self.config = Some(config);
    }
    
    pub fn get_config(&self) -> Option<&EmailConfig> {
        self.config.as_ref()
    }
    
    pub async fn send_export_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        job_name: &str,
        export_format: &str,
    ) -> Result<()> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow!("Email configuration not set"))?;
        
        info!("Sending export file: {:?}", file_path.as_ref());
        
        let file_path = file_path.as_ref();
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;
        
        let file_content = std::fs::read(file_path)
            .map_err(|e| anyhow!("Failed to read export file: {}", e))?;
        
        let content_type = match export_format.to_lowercase().as_str() {
            "csv" => "text/csv",
            "json" => "application/json",
            "pdf" => "application/pdf",
            _ => "application/octet-stream",
        };
        
        let attachment = Attachment::new(file_name.to_string())
            .body(file_content, content_type.parse().unwrap());
        
        let email = Message::builder()
            .from(config.sender_email.parse()
                .map_err(|e| anyhow!("Invalid sender email: {}", e))?)
            .to(config.receiver_email.parse()
                .map_err(|e| anyhow!("Invalid receiver email: {}", e))?)
            .subject(format!("Web Scraping Export: {} ({})", job_name, export_format.to_uppercase()))
            .multipart(
                MultiPart::mixed()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "Hello,\n\nPlease find attached the export file for the web scraping job '{}'.\n\nExport Details:\n- Job Name: {}\n- Format: {}\n- File: {}\n- Generated: {}\n\nBest regards,\nAutomated Web Scraper",
                                job_name,
                                job_name,
                                export_format.to_uppercase(),
                                file_name,
                                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                            ))
                    )
                    .singlepart(attachment)
            )
            .map_err(|e| anyhow!("Failed to build email: {}", e))?;
        
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        
        let mailer = if config.use_tls {
            SmtpTransport::relay(&config.smtp_server)?
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.smtp_server)
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };
        
        match mailer.send(&email) {
            Ok(_) => {
                info!("Email sent successfully to {}", config.receiver_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email: {}", e);
                Err(anyhow!("Failed to send email: {}", e))
            }
        }
    }
    
    pub async fn send_notification(
        &self,
        subject: &str,
        message: &str,
    ) -> Result<()> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow!("Email configuration not set"))?;
        
        info!("Sending notification email: {}", subject);
        
        let email = Message::builder()
            .from(config.sender_email.parse()
                .map_err(|e| anyhow!("Invalid sender email: {}", e))?)
            .to(config.receiver_email.parse()
                .map_err(|e| anyhow!("Invalid receiver email: {}", e))?)
            .subject(subject)
            .body(message.to_string())
            .map_err(|e| anyhow!("Failed to build email: {}", e))?;
        
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        
        let mailer = if config.use_tls {
            SmtpTransport::relay(&config.smtp_server)?
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.smtp_server)
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };
        
        match mailer.send(&email) {
            Ok(_) => {
                info!("Notification email sent successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to send notification email: {}", e);
                Err(anyhow!("Failed to send notification email: {}", e))
            }
        }
    }
    
    pub async fn test_connection(&self) -> Result<()> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow!("Email configuration not set"))?;
        
        info!("Testing email connection to {}", config.smtp_server);
        
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        
        let mailer = if config.use_tls {
            SmtpTransport::relay(&config.smtp_server)?
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.smtp_server)
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };
        
        // Test connection by checking if we can connect
        match mailer.test_connection() {
            Ok(true) => {
                info!("Email connection test successful");
                Ok(())
            }
            Ok(false) => {
                error!("Email connection test failed: Unable to connect");
                Err(anyhow!("Unable to connect to SMTP server"))
            }
            Err(e) => {
                error!("Email connection test failed: {}", e);
                Err(anyhow!("SMTP connection error: {}", e))
            }
        }
    }
    
    pub fn validate_email_address(email: &str) -> Result<()> {
        email.parse::<lettre::Address>()
            .map_err(|e| anyhow!("Invalid email address '{}': {}", email, e))?;
        Ok(())
    }
    
    pub fn validate_config(config: &EmailConfig) -> Result<()> {
        // Validate email addresses
        Self::validate_email_address(&config.sender_email)?;
        Self::validate_email_address(&config.receiver_email)?;
        
        // Validate SMTP server
        if config.smtp_server.is_empty() {
            return Err(anyhow!("SMTP server cannot be empty"));
        }
        
        // Validate port
        if config.smtp_port == 0 {
            return Err(anyhow!("Invalid SMTP port: {}", config.smtp_port));
        }
        
        // Validate credentials
        if config.username.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }
        
        if config.password.is_empty() {
            return Err(anyhow!("Password cannot be empty"));
        }
        
        Ok(())
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new()
    }
}

// Common SMTP configurations for popular email providers
pub struct SmtpPresets;

impl SmtpPresets {
    pub fn gmail() -> (String, u16, bool) {
        ("smtp.gmail.com".to_string(), 587, true)
    }
    
    pub fn outlook() -> (String, u16, bool) {
        ("smtp-mail.outlook.com".to_string(), 587, true)
    }
    
    pub fn yahoo() -> (String, u16, bool) {
        ("smtp.mail.yahoo.com".to_string(), 587, true)
    }
    
    pub fn custom(server: &str, port: u16, use_tls: bool) -> (String, u16, bool) {
        (server.to_string(), port, use_tls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        assert!(EmailService::validate_email_address("test@example.com").is_ok());
        assert!(EmailService::validate_email_address("user.name+tag@domain.co.uk").is_ok());
        
        assert!(EmailService::validate_email_address("invalid-email").is_err());
        assert!(EmailService::validate_email_address("@domain.com").is_err());
        assert!(EmailService::validate_email_address("user@").is_err());
    }
    
    #[test]
    fn test_config_validation() {
        let valid_config = EmailConfig {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: "user@gmail.com".to_string(),
            password: "password".to_string(),
            sender_email: "sender@gmail.com".to_string(),
            receiver_email: "receiver@gmail.com".to_string(),
            use_tls: true,
        };
        
        assert!(EmailService::validate_config(&valid_config).is_ok());
        
        let invalid_config = EmailConfig {
            smtp_server: "".to_string(),
            smtp_port: 0,
            username: "".to_string(),
            password: "".to_string(),
            sender_email: "invalid-email".to_string(),
            receiver_email: "invalid-email".to_string(),
            use_tls: true,
        };
        
        assert!(EmailService::validate_config(&invalid_config).is_err());
    }
    
    #[test]
    fn test_smtp_presets() {
        let (server, port, tls) = SmtpPresets::gmail();
        assert_eq!(server, "smtp.gmail.com");
        assert_eq!(port, 587);
        assert!(tls);
        
        let (server, port, tls) = SmtpPresets::outlook();
        assert_eq!(server, "smtp-mail.outlook.com");
        assert_eq!(port, 587);
        assert!(tls);
    }
}