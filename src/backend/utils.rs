use web_sys::window;



#[cfg(feature = "server")]
pub mod server_utils {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};
    use base64::{engine::general_purpose, Engine as _};
    use rand::RngCore;
    use hex;
    use dotenvy::dotenv;
    use sha2::{Digest, Sha256};

    fn get_key() -> [u8; 32] {
        dotenv().ok();
        let key = std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
        let key_bytes = hex::decode(key).expect("ENCRYPTION_KEY must be a valid hex string");
        key_bytes
            .try_into()
            .expect("ENCRYPTION_KEY must be exactly 32 bytes")
    }

    pub fn encrypt(plain: &str) -> String {
        let key_bytes = get_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let hash = Sha256::digest(plain.as_bytes());
        let nonce_bytes = &hash[..12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plain.as_bytes()).expect("encryption failure");

        let combined = [&nonce_bytes[..], &ciphertext[..]].concat();

        general_purpose::STANDARD.encode(&combined)
    }

    pub fn decrypt(encoded: &str) -> Result<String, aes_gcm::Error> {
        let key_bytes = get_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let combined = general_purpose::STANDARD.decode(encoded).expect("Invalid base64");

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = cipher.decrypt(nonce, ciphertext)?;

        Ok(String::from_utf8(plaintext_bytes).expect("Invalid UTF-8"))
    }
}

pub type ThemeModes = (String, String);
pub const CURRENT_THEME: &str = "current_theme";
const LIGHT_THEME: &str = "light";
const DARK_THEME: &str = "dark";

fn get_system_theme_mode() -> String {
    let modes: ThemeModes = (String::from(LIGHT_THEME),String::from(DARK_THEME));
    let is_dark = window()
        .and_then(|win| win.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false);
    if is_dark { modes.1 } else { modes.0 }
}

pub fn read_local_storage_value(key:&str) -> String{
    let local_storage = window()
        .and_then(|win| win.local_storage().ok().flatten());

    local_storage
        .and_then(|storage| storage.get(key).ok().flatten())
        .unwrap_or_else(|| get_system_theme_mode())
}

use phonenumber::{parse,country};


pub fn format_phone_number(input: &str) -> String{
    if let Ok(num) = parse(Some(country::TZ), input) {
        //num.format().mode(phonenumber::Mode::National).to_string()
        num.format().mode(phonenumber::Mode::International).to_string()
    } else {
        input.to_string()
    }
}