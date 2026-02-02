#!/usr/bin/env python3
"""
Dis Email Service
Sends encrypted email verification with anonymization
"""

import os
import smtplib
import hashlib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from cryptography.fernet import Fernet
from typing import Optional
import logging

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class EmailEncryption:
    """Handle email encryption/decryption"""
    
    def __init__(self, encryption_key: Optional[str] = None):
        """Initialize with encryption key"""
        if encryption_key:
            # Use provided key (base64 encoded)
            self.fernet = Fernet(encryption_key.encode())
        else:
            # Generate new key
            key = Fernet.generate_key()
            self.fernet = Fernet(key)
            logger.warning(f"Generated new encryption key: {key.decode()}")
    
    def encrypt_email(self, email: str) -> str:
        """Encrypt an email address"""
        encrypted = self.fernet.encrypt(email.encode())
        return encrypted.decode()
    
    def decrypt_email(self, encrypted_email: str) -> str:
        """Decrypt an email address"""
        decrypted = self.fernet.decrypt(encrypted_email.encode())
        return decrypted.decode()
    
    @staticmethod
    def hash_email(email: str) -> str:
        """Create SHA-256 hash for email lookup"""
        return hashlib.sha256(email.encode()).hexdigest()


class EmailSender:
    """Send verification emails"""
    
    def __init__(self):
        """Initialize email sender with SMTP config"""
        self.smtp_host = os.getenv('SMTP_HOST', 'localhost')
        self.smtp_port = int(os.getenv('SMTP_PORT', '587'))
        self.smtp_username = os.getenv('SMTP_USERNAME', '')
        self.smtp_password = os.getenv('SMTP_PASSWORD', '')
        self.smtp_from = os.getenv('SMTP_FROM', 'no-reply@example.com')
        self.smtp_use_tls = os.getenv('SMTP_USE_TLS', 'true').lower() == 'true'
        
        # Encryption
        encryption_key = os.getenv('EMAIL_ENCRYPTION_KEY')
        self.encryption = EmailEncryption(encryption_key)
    
    def load_template(self, template_name: str) -> str:
        """Load email template"""
        template_path = os.path.join(
            os.path.dirname(__file__),
            'templates',
            f'{template_name}.html'
        )
        with open(template_path, 'r') as f:
            return f.read()
    
    def send_verification(
        self,
        to_email: str,
        verification_code: str,
        domain: str
    ) -> bool:
        """Send verification email"""
        try:
            # Load template
            template = self.load_template('verify')
            
            # Replace placeholders
            html_content = template.replace('{{verification_code}}', verification_code)
            html_content = html_content.replace('{{domain}}', domain)
            
            # Create message
            msg = MIMEMultipart('alternative')
            msg['Subject'] = 'Verify Your Account - Dis'
            msg['From'] = self.smtp_from
            msg['To'] = to_email
            
            # Attach HTML
            msg.attach(MIMEText(html_content, 'html'))
            
            # Send email
            with smtplib.SMTP(self.smtp_host, self.smtp_port) as server:
                if self.smtp_use_tls:
                    server.starttls()
                
                if self.smtp_username and self.smtp_password:
                    server.login(self.smtp_username, self.smtp_password)
                
                server.send_message(msg)
            
            logger.info(f"Verification email sent to {self.encryption.hash_email(to_email)[:8]}...")
            return True
            
        except Exception as e:
            logger.error(f"Failed to send email: {e}")
            return False
    
    def send_recovery(
        self,
        to_email: str,
        recovery_code: str,
        domain: str
    ) -> bool:
        """Send account recovery email"""
        try:
            # Similar to verification, but with recovery template
            # For brevity, using inline content
            html_content = f"""
            <html>
                <body>
                    <h2>Account Recovery - Dis</h2>
                    <p>Your recovery code is:</p>
                    <h3>{recovery_code}</h3>
                    <p>This code will expire in 1 hour.</p>
                    <p>If you didn't request this, please ignore this email.</p>
                </body>
            </html>
            """
            
            msg = MIMEMultipart('alternative')
            msg['Subject'] = 'Account Recovery - Dis'
            msg['From'] = self.smtp_from
            msg['To'] = to_email
            msg.attach(MIMEText(html_content, 'html'))
            
            with smtplib.SMTP(self.smtp_host, self.smtp_port) as server:
                if self.smtp_use_tls:
                    server.starttls()
                
                if self.smtp_username and self.smtp_password:
                    server.login(self.smtp_username, self.smtp_password)
                
                server.send_message(msg)
            
            logger.info(f"Recovery email sent to {self.encryption.hash_email(to_email)[:8]}...")
            return True
            
        except Exception as e:
            logger.error(f"Failed to send recovery email: {e}")
            return False


def main():
    """Main entry point"""
    sender = EmailSender()
    
    # Example usage
    test_email = os.getenv('TEST_EMAIL')
    domain = os.getenv('DOMAIN', 'example.com')
    
    if test_email:
        logger.info("Sending test email...")
        success = sender.send_verification(test_email, '123456', domain)
        if success:
            logger.info("Test email sent successfully")
        else:
            logger.error("Failed to send test email")


if __name__ == '__main__':
    main()
