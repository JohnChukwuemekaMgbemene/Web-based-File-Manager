use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub username: String,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct AuthManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    valid_credentials: HashMap<String, String>,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut credentials = HashMap::new();
        credentials.insert("admin".to_string(), "admin123".to_string());
        
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            valid_credentials: credentials,
        }
    }
    
    pub fn authenticate(&self, username: &str, password: &str) -> Option<String> {
        if let Some(stored_password) = self.valid_credentials.get(username) {
            if stored_password == password {
                // Generate session token
                let token = format!("{}{}", username, 
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                
                let session = Session {
                    username: username.to_string(),
                    created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    expires_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600, // 1 hour
                };
                
                if let Ok(mut sessions) = self.sessions.lock() {
                    sessions.insert(token.clone(), session);
                }
                
                return Some(token);
            }
        }
        None
    }
    
    pub fn is_valid_token(&self, token: &str) -> bool {
        if let Ok(sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get(token) {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                return session.expires_at > now;
            }
        }
        false
    }
    
    pub fn logout(&self, token: &str) -> bool {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(token).is_some()
        } else {
            false
        }
    }
    
    pub fn invalidate_token(&self, token: &str) -> bool {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(token).is_some()
        } else {
            false
        }
    }
    
    pub fn cleanup_expired_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            sessions.retain(|_, session| session.expires_at > now);
        }
    }
    
    pub fn get_session_info(&self, token: &str) -> Option<Session> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.get(token).cloned()
        } else {
            None
        }
    }
}

pub fn generate_login_html() -> String {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Login - FirxTTech Solutions</title>
        <meta charset="UTF-8">
        <style>
            body { 
                font-family: Arial, sans-serif; 
                background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); 
                margin: 0; 
                padding: 0; 
                min-height: 100vh; 
                display: flex; 
                align-items: center; 
                justify-content: center; 
            }
            .login-container { 
                background: rgba(255, 255, 255, 0.95); 
                padding: 40px; 
                border-radius: 16px; 
                box-shadow: 0 8px 32px rgba(0,0,0,0.15); 
                max-width: 400px; 
                width: 100%; 
                backdrop-filter: blur(10px); 
                border: 1px solid rgba(255, 255, 255, 0.2); 
            }
            .logo { 
                text-align: center; 
                margin-bottom: 30px; 
            }
            .logo h1 { 
                color: #1f2937; 
                font-size: 28px; 
                margin: 0; 
                background: linear-gradient(135deg, #f97316, #2563eb); 
                -webkit-background-clip: text; 
                -webkit-text-fill-color: transparent; 
                background-clip: text; 
            }
            .logo p { 
                color: #6b7280; 
                margin: 5px 0 0 0; 
                font-size: 14px; 
            }
            .form-group { 
                margin-bottom: 20px; 
            }
            label { 
                display: block; 
                margin-bottom: 8px; 
                color: #374151; 
                font-weight: 600; 
            }
            input[type="text"], input[type="password"] { 
                width: 100%; 
                padding: 12px; 
                border: 2px solid #e5e7eb; 
                border-radius: 8px; 
                font-size: 16px; 
                transition: all 0.3s ease; 
                box-sizing: border-box; 
            }
            input[type="text"]:focus, input[type="password"]:focus { 
                outline: none; 
                border-color: #2563eb; 
                box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1); 
            }
            .login-btn { 
                width: 100%; 
                padding: 12px; 
                background: linear-gradient(135deg, #2563eb, #1d4ed8); 
                color: white; 
                border: none; 
                border-radius: 8px; 
                font-size: 16px; 
                font-weight: 600; 
                cursor: pointer; 
                transition: all 0.3s ease; 
                box-shadow: 0 4px 15px rgba(37, 99, 235, 0.2); 
            }
            .login-btn:hover { 
                background: linear-gradient(135deg, #1d4ed8, #1e40af); 
                transform: translateY(-2px); 
                box-shadow: 0 8px 25px rgba(37, 99, 235, 0.3); 
            }
            .error { 
                color: #ef4444; 
                font-size: 14px; 
                margin-top: 10px; 
                text-align: center; 
            }
            .credentials { 
                background: rgba(249, 115, 22, 0.1); 
                border: 1px solid rgba(249, 115, 22, 0.2); 
                padding: 15px; 
                border-radius: 8px; 
                margin-bottom: 20px; 
            }
            .credentials h3 { 
                margin: 0 0 10px 0; 
                color: #ea580c; 
                font-size: 14px; 
            }
            .credentials p { 
                margin: 5px 0; 
                font-family: monospace; 
                font-size: 13px; 
                color: #9a3412; 
            }
        </style>
    </head>
    <body>
        <div class="login-container">
            <div class="logo">
                <h1>🦀 File Manager</h1>
                <p>FirxTTech Solutions</p>
            </div>
            
            <div class="credentials">
                <h3>🔑 Demo Credentials</h3>
                <p>👤 Username: admin</p>
                <p>🔐 Password: admin123</p>
            </div>
            
            <form method="POST" action="/login">
                <div class="form-group">
                    <label for="username">👤 Username</label>
                    <input type="text" id="username" name="username" required value="admin">
                </div>
                <div class="form-group">
                    <label for="password">🔐 Password</label>
                    <input type="password" id="password" name="password" required value="admin123">
                </div>
                <button type="submit" class="login-btn">🚀 Login</button>
            </form>
        </div>
    </body>
    </html>
    "#.to_string()
}