use dioxus::signals::Signal;
use serde::Serialize;
use serde::Deserialize;


#[derive(Serialize,Deserialize)]
pub struct Token {
    token: String
}

impl Token {
    pub fn get_token(&self) -> String {
        self.token.clone()
    }

    pub fn new (value:String) -> Token {
        Token {
            token: value
        }
    }
}

use dioxus::hooks::Resource;

#[derive(Clone)]
pub struct ResourceValues {
    pub is_authenticated: Resource<Result<Token,String>>,
    pub theme_mode: Signal<String>
}

#[cfg(feature="server")] 
pub mod Forms {
    use validator::Validate;
    use validator::ValidationError;
    use serde::Serialize;
    use serde::Deserialize;

    // #[derive(Serialize,Deserialize)]
    // pub struct Token {
    //     token: String
    // }

    #[derive(Clone, Debug,Serialize,Deserialize)]
    pub enum UserRole {
        Buyer,
        Seller,
    }

    fn validate_role(role: &UserRole) -> Result<(), ValidationError> {
        match role {
            UserRole::Buyer | UserRole::Seller => Ok(()),
            _ => Err(ValidationError::new("invalid_role")),
        }
    }
    

    #[derive(Serialize,Deserialize,Debug,Clone, Validate)]
    pub struct AuthUserForm {
        #[validate(length(min = 10, max = 14, message = "Phone number must be 14 characters"))]
        password: String,
        #[validate(custom(function = "validate_role", message = "Role must be either Buyer or Seller"))]
        role: Option<UserRole>
    }

    impl AuthUserForm {
        pub fn create_buyer(password: String) -> AuthUserForm {
            AuthUserForm {
                password: password,
                role: Some(UserRole::Buyer)
            }
        }

        pub fn new(password:String) -> AuthUserForm {
            AuthUserForm { password: password, role: None }
        }

        pub fn get_password(&self) -> String {
            self.password.clone()
        }
        pub fn get_role(&self) -> Option<UserRole> {
            self.role.clone()
        }
    }
}