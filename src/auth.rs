use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct AuthManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    users: Arc<Mutex<HashMap<String, User>>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

impl AuthManager {
    pub fn new() -> Self {
        let auth = AuthManager { // Remove mut
            sessions: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Add default admin user (password: "admin123")
        auth.add_user("admin", "admin123");
        auth.add_user("Firxt", "377490")
        auth
    }
    
    pub fn add_user(&self, username: &str, password: &str) {
        let password_hash = self.hash_password(password);
        let user = User {
            username: username.to_string(),
            password_hash,
        };
        
        let mut users = self.users.lock().unwrap();
        users.insert(username.to_string(), user);
    }
    
    pub fn authenticate(&self, username: &str, password: &str) -> Option<String> {
        let users = self.users.lock().unwrap();
        if let Some(user) = users.get(username) {
            if self.verify_password(password, &user.password_hash) {
                let session_id = Uuid::new_v4().to_string();
                let expires_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + 3600; // 1 hour
                
                let session = Session {
                    user_id: username.to_string(),
                    expires_at,
                };
                
                let mut sessions = self.sessions.lock().unwrap();
                sessions.insert(session_id.clone(), session);
                
                return Some(session_id);
            }
        }
        None
    }
    
    pub fn validate_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if session.expires_at > now {
                return true;
            } else {
                // Remove expired session
                sessions.remove(session_id);
            }
        }
        false
    }
    
    pub fn logout(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id);
    }
    
    // Simple password hashing (use bcrypt in production)
    fn hash_password(&self, password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }
}

pub fn generate_login_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Login - File Manager</title>
    <meta charset="UTF-8">
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 0; 
            padding: 0; 
            background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); 
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
        }
        .login-container {
            background: rgba(255, 255, 255, 0.95);
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            backdrop-filter: blur(10px);
            width: 100%;
            max-width: 400px;
        }
        h1 {
            text-align: center;
            color: #1f2937;
            margin-bottom: 30px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 5px;
            color: #374151;
            font-weight: 500;
        }
        input[type="text"], input[type="password"] {
            width: 100%;
            padding: 12px;
            border: 2px solid #e5e7eb;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.3s;
            box-sizing: border-box; /* Add this to fix the width issue */
        }
        input[type="text"]:focus, input[type="password"]:focus {
            outline: none;
            border-color: #2563eb;
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
            transition: all 0.3s;
            box-sizing: border-box; /* Add this for consistency */
        }
        .login-btn:hover {
            background: linear-gradient(135deg, #1d4ed8, #1e40af);
            transform: translateY(-2px);
            box-shadow: 0 4px 20px rgba(37, 99, 235, 0.3);
        }
        .error {
            color: #dc2626;
            text-align: center;
            margin-top: 15px;
            font-size: 14px;
        }
        .default-creds {
            background: rgba(249, 115, 22, 0.1);
            border: 1px solid rgba(249, 115, 22, 0.3);
            border-radius: 8px;
            padding: 15px;
            margin-top: 20px;
            font-size: 14px;
            color: #ea580c;
        }
    </style>
</head>
<body>
    <div class="login-container">
        <h1>🔒 File Manager Login</h1>
        <form method="POST" action="/login">
            <div class="form-group">
                <label for="username">Username:</label>
                <input type="text" id="username" name="username" required>
            </div>
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit" class="login-btn">Login</button>
        </form>
        <div class="default-creds">
            <strong>Default credentials:</strong><br>
            Username: admin<br>
            Password: admin123
        </div>
    </div>
</body>
</html>"#.to_string()
}