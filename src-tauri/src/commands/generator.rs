use rand_core::{OsRng, RngCore};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*,?";
const DEFAULT_PASSWORD_LEN: usize = 12;
const DEFAULT_SYMBOL_COUNT: usize = 2;

#[tauri::command]
pub fn generate_password() -> Result<String, String> {
    generate_password_value()
}

fn generate_password_value() -> Result<String, String> {
    let mut non_symbols = Vec::new();
    non_symbols.extend_from_slice(LOWER);
    non_symbols.extend_from_slice(UPPER);
    non_symbols.extend_from_slice(DIGITS);

    let mut password = Vec::with_capacity(DEFAULT_PASSWORD_LEN);
    password.push(random_char(LOWER));
    password.push(random_char(UPPER));
    password.push(random_char(DIGITS));
    for _ in 0..DEFAULT_SYMBOL_COUNT {
        password.push(random_char(SYMBOLS));
    }
    while password.len() < DEFAULT_PASSWORD_LEN {
        password.push(random_char(&non_symbols));
    }
    shuffle(&mut password);

    String::from_utf8(password).map_err(|_| "generated password is not valid UTF-8".to_string())
}

fn random_char(chars: &[u8]) -> u8 {
    chars[random_index(chars.len())]
}

fn random_index(len: usize) -> usize {
    let len = len as u32;
    let zone = u32::MAX - (u32::MAX % len);
    loop {
        let value = OsRng.next_u32();
        if value < zone {
            return (value % len) as usize;
        }
    }
}

fn shuffle(chars: &mut [u8]) {
    for i in (1..chars.len()).rev() {
        let j = random_index(i + 1);
        chars.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_password_uses_default_rule() {
        let password = generate_password_value().unwrap();
        let specials = "!@#$%^&*,?";
        let special_count = password.chars().filter(|c| specials.contains(*c)).count();

        assert_eq!(password.chars().count(), 12);
        assert_eq!(special_count, 2);
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || specials.contains(c)));
    }

    #[test]
    fn generated_password_uses_only_configured_special_characters() {
        for _ in 0..50 {
            let password = generate_password_value().unwrap();

            assert!(!password.chars().any(|c| "()_-+=[]{};:.".contains(c)));
        }
    }
}
