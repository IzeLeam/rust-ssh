pub mod database {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    #[derive(Serialize, Deserialize)]
    pub struct User {
        pub username: String,
        pub password: String,
        pub pubkey: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct Database {
        pub users: HashMap<String, User>,
    }

    const DB_FILE: &str = "users.json";

    impl User {
        pub fn _new(username: String, password: String, pubkey: String) -> Self {
            User {
                username,
                password,
                pubkey,
            }
        }
    }

    impl Database {
        pub fn new() -> Self {
            if File::open(DB_FILE).is_err() {
                File::create(DB_FILE).expect("Erreur de création de la base de données");
            }
            Database {
                users: HashMap::new(),
            }
        }

        pub fn add_user(&mut self, username: String, password: String, pubkey: String) -> Result<(), String> {
            if self.users.contains_key(&username) {
                return Err(String::from("L'utilisateur existe déjà"));
            }
            let user = User {
                username: username.clone(),
                password,
                pubkey,
            };
            self.users.insert(username, user);
            Ok(())
        }

        pub fn _remove_user(&mut self, username: &str) {
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

    #[test]
    fn test_add_user() {
        let mut db = Database::new();
        let username = "testuser".to_string();
        let password = "testpassword".to_string();
        let pubkey = "testpubkey".to_string();
        db.add_user(username.clone(), password.clone(), pubkey.clone()).unwrap();
        let user = db.get_user(&username).unwrap();
        assert_eq!(user.username, username);
        assert_eq!(user.password, password);
        assert_eq!(user.pubkey, pubkey);
    }
    #[test]
    fn test_remove_user() {
        let mut db = Database::new();
        let username = "testuser".to_string();
        let password = "testpassword".to_string();
        let pubkey = "testpubkey".to_string();
        db.add_user(username.clone(), password.clone(), pubkey.clone()).unwrap();
        db._remove_user(&username);
        assert!(db.get_user(&username).is_none());
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
        let username = "testuser".to_string();
        let password = "testpassword".to_string();
        let pubkey = "testpubkey".to_string();
        db.add_user(username.clone(), password.clone(), pubkey.clone()).unwrap();
        db.save_users();
        let mut new_db = Database::new();
        new_db.load_users();
        assert_eq!(new_db.get_user(&username).unwrap().username, username);
    }
}
