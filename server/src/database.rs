pub mod database {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    use::common::auth::AuthMethod;

    #[derive(Serialize, Deserialize)]
    pub struct User {
        pub username: String,
        pub secret: String,
        pub auth_method: AuthMethod,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Database {
        pub users: HashMap<String, User>,
    }

    const DB_FILE: &str = "users.json";

    impl User {
        pub fn new(username: String, secret: String, auth_method: AuthMethod) -> Self {
            User {
                username,
                secret,
                auth_method,
            }
        }
    }

    impl Database {
        pub fn new() -> Self {
            Database {
                users: HashMap::new(),
            }
        }

        pub fn add_user(&mut self, username: String, secret: String, auth_method: AuthMethod) -> Result<(), String> {
            if self.users.contains_key(&username) {
                return Err(String::from("L'utilisateur existe déjà"));
            }
            let user = User {
                username: username.clone(),
                secret,
                auth_method,
            };
            self.users.insert(username, user);
            Ok(())
        }

        pub fn remove_user(&mut self, username: &str) {
            self.users.remove(username);
        }

        pub fn get_user(&self, username: &str) -> Option<&User> {
            self.users.get(username)
        }

        pub fn load_users(&mut self) {
            File::open(DB_FILE)
                .and_then(|file| {
                    let reader = BufReader::new(file);
                    let users: HashMap<String, User> = serde_json::from_reader(reader)?;
                    self.users.extend(users);
                    Ok(())
                })
                .unwrap_or_else(|err| {
                    eprintln!("Erreur lors du chargement des utilisateurs : {}", err);
                });
        }

        pub fn save_users(&self) {
            File::create(DB_FILE)
                .and_then(|file| {
                    let writer = BufWriter::new(file);
                    serde_json::to_writer(writer, &self.users)?;
                    Ok(())
                })
                .unwrap_or_else(|err| {
                    eprintln!("Erreur lors de la sauvegarde des utilisateurs : {}", err);
                });
        }
        
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use database::Database;
    use common::auth::AuthMethod;

    #[test]
    fn test_add_user() {
        let mut db = Database::new();
        db.add_user("user1".to_string(), "password1".to_string(), AuthMethod::Password).unwrap();
        assert_eq!(db.users.len(), 1);
        assert!(db.users.contains_key("user1"));
        assert_eq!(db.users["user1"].secret, "password1");
        assert_eq!(db.users["user1"].auth_method, AuthMethod::Password);
    }
    #[test]
    fn test_remove_user() {
        let mut db = Database::new();
        db.add_user("user1".to_string(), "password1".to_string(), AuthMethod::Password).unwrap();
        db.remove_user("user1");
        assert_eq!(db.users.len(), 0);
        assert!(!db.users.contains_key("user1"));
    }
    #[test]
    fn test_get_user() {
        let mut db = Database::new();
        db.add_user("user1".to_string(), "password1".to_string(), AuthMethod::Password).unwrap();
        let user = db.get_user("user1").unwrap();
        assert_eq!(user.username, "user1");
        assert_eq!(user.secret, "password1");
        assert_eq!(user.auth_method, AuthMethod::Password);
    }
    #[test]
    fn test_load_users() {
        let mut db = Database::new();
        db.load_users();
        assert!(!db.users.is_empty());
    }
    #[test]
    fn test_save_users() {
        let mut db = Database::new();
        db.add_user("user1".to_string(), "password1".to_string(), AuthMethod::Password).unwrap();
        db.save_users();
        let mut db2 = Database::new();
        db2.load_users();
        assert_eq!(db2.users.len(), 1);
        assert!(db2.users.contains_key("user1"));
        assert_eq!(db2.users["user1"].secret, "password1");
        assert_eq!(db2.users["user1"].auth_method, AuthMethod::Password);
    }
    
}
