//! Masks credentials that might end up in log lines (for example inside an
//! HTTP error body echoed back by a provider). Defense in depth: code must not
//! log secrets in the first place.

use std::borrow::Cow;

/// Prefixes of well-known credential formats. The prefix stays visible so logs
/// still show which kind of credential was involved.
const PREFIXES: &[&str] = &[
    "sk-",     // OpenAI, Anthropic (sk-ant-), OpenRouter (sk-or-)
    "AIza",    // Google API keys
    "gsk_",    // Groq
    "xoxb-",   // Slack bot tokens
    "xoxp-",   // Slack user tokens
    "ntn_",    // Notion integration tokens
    "secret_", // Notion legacy tokens
    "Bearer ",
    "hooks.slack.com/services/",
];

/// Minimum length of the part after the prefix for it to count as a credential,
/// so ordinary words such as "sk-learn" are not mangled.
const MIN_SECRET_LEN: usize = 12;

const MASK: &str = "[REDACTED]";

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '+' | '=')
}

pub fn redact_secrets(input: &str) -> Cow<'_, str> {
    if !PREFIXES.iter().any(|p| input.contains(p)) {
        return Cow::Borrowed(input);
    }

    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while !rest.is_empty() {
        let next = PREFIXES
            .iter()
            .filter_map(|p| rest.find(p).map(|i| (i, *p)))
            .min_by_key(|(i, _)| *i);
        let Some((start, prefix)) = next else {
            out.push_str(rest);
            break;
        };
        let after_prefix = start + prefix.len();
        let token_len = rest[after_prefix..]
            .find(|c: char| !is_token_char(c))
            .unwrap_or(rest.len() - after_prefix);

        out.push_str(&rest[..after_prefix]);
        if token_len >= MIN_SECRET_LEN {
            out.push_str(MASK);
        } else {
            out.push_str(&rest[after_prefix..after_prefix + token_len]);
        }
        rest = &rest[after_prefix + token_len..];
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_clean_text_untouched() {
        assert!(matches!(
            redact_secrets("enhancing meeting 42"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn masks_provider_keys() {
        let line = r#"{"error":"invalid key sk-ant-api03-AbCdEf1234567890xyz","next":"ok"}"#;
        let red = redact_secrets(line);
        assert_eq!(red, r#"{"error":"invalid key sk-[REDACTED]","next":"ok"}"#);
    }

    #[test]
    fn masks_multiple_and_bearer() {
        let line = "Authorization: Bearer abcdefghijklmnop12 and AIzaSyA1234567890abcdef";
        assert_eq!(
            redact_secrets(line),
            "Authorization: Bearer [REDACTED] and AIza[REDACTED]"
        );
    }

    #[test]
    fn masks_slack_webhook_path() {
        let url = "https://hooks.slack.com/services/T000/B000/XXXXXXXXXXXXXXXX";
        assert_eq!(
            redact_secrets(url),
            "https://hooks.slack.com/services/[REDACTED]"
        );
    }

    #[test]
    fn keeps_short_lookalikes() {
        assert_eq!(
            redact_secrets("uses sk-learn for tests"),
            "uses sk-learn for tests"
        );
    }
}
