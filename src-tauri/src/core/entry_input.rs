#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEntryInput {
    pub name: String,
    pub url: String,
    pub description: String,
    pub alias: String,
    pub account: String,
    pub password: String,
    pub tags: String,
}

pub fn normalize_create_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    account: Option<String>,
    password: String,
    tags: Option<String>,
) -> Result<NormalizedEntryInput, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("name is required".to_string());
    }

    if password.is_empty() {
        return Err("password is required".to_string());
    }

    Ok(NormalizedEntryInput {
        name,
        url: trim_optional(url),
        description: trim_optional(description),
        alias: trim_optional(alias),
        account: trim_optional(account),
        password,
        tags: trim_optional(tags),
    })
}

pub fn normalize_update_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tags: Option<String>,
) -> Result<NormalizedEntryInput, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("name is required".to_string());
    }

    Ok(NormalizedEntryInput {
        name,
        url: trim_optional(url),
        description: trim_optional(description),
        alias: trim_optional(alias),
        account: trim_optional(account),
        password: password.unwrap_or_default(),
        tags: trim_optional(tags),
    })
}

fn trim_optional(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_create_entry_rejects_blank_name() {
        let result = normalize_create_entry(
            "   ".to_string(),
            Some("https://example.com".to_string()),
            None,
            None,
            None,
            "secret".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "name is required");
    }

    #[test]
    fn normalize_create_entry_rejects_empty_password() {
        let result = normalize_create_entry(
            "Example".to_string(),
            None,
            None,
            None,
            None,
            "".to_string(),
            None,
        );

        assert_eq!(result.unwrap_err(), "password is required");
    }

    #[test]
    fn normalize_create_entry_trims_metadata() {
        let input = normalize_create_entry(
            "  GitHub  ".to_string(),
            Some("  https://github.com  ".to_string()),
            Some("  Developer account  ".to_string()),
            Some("  work login  ".to_string()),
            Some("  octocat  ".to_string()),
            "  secret  ".to_string(),
            Some("  dev,code  ".to_string()),
        )
        .unwrap();

        assert_eq!(input.name, "GitHub");
        assert_eq!(input.url, "https://github.com");
        assert_eq!(input.description, "Developer account");
        assert_eq!(input.alias, "work login");
        assert_eq!(input.account, "octocat");
        assert_eq!(input.password, "  secret  ");
        assert_eq!(input.tags, "dev,code");
    }

    #[test]
    fn normalize_update_entry_allows_omitted_password() {
        let input = normalize_update_entry(
            "  GitHub  ".to_string(),
            Some("  https://github.com  ".to_string()),
            None,
            None,
            Some("  octocat  ".to_string()),
            None,
            Some("  dev  ".to_string()),
        )
        .unwrap();

        assert_eq!(input.name, "GitHub");
        assert_eq!(input.url, "https://github.com");
        assert_eq!(input.account, "octocat");
        assert_eq!(input.password, "");
        assert_eq!(input.tags, "dev");
    }

    #[test]
    fn normalize_update_entry_rejects_blank_name() {
        let result = normalize_update_entry(
            "   ".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(result.unwrap_err(), "name is required");
    }
}
