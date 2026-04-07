use regex::Regex;
use entropy::shannon_entropy;
use std::sync::OnceLock;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: Regex,
    pub capture_group: usize,
}

static PATTERNS: OnceLock<Vec<SecretPattern>> = OnceLock::new();

pub fn get_secret_patterns() -> &'static Vec<SecretPattern> {
    PATTERNS.get_or_init(|| {
        vec![
            SecretPattern { name: "Private Key", regex: Regex::new(r"-----BEGIN [a-zA-Z\s]+ PRIVATE KEY-----").expect("Static Regex"), capture_group: 0 },
            SecretPattern { name: "AWS Access Key", regex: Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").expect("Static Regex"), capture_group: 1 },
            SecretPattern { name: "Auth Header", regex: Regex::new(r"(?i)Authorization:\s*(?:Bearer|Basic)\s+([a-zA-Z0-9_=\-\.]{16,})").expect("Static Regex"), capture_group: 1 },
            SecretPattern { name: "Platform Token", regex: Regex::new(r"(ghp_|xox[bap]-|npm_|rk_live_|sk_live_|AIzaSy)[a-zA-Z0-9_\-]{20,}").expect("Static Regex"), capture_group: 0 },
            SecretPattern { name: "API Key/Secret", regex: Regex::new(r"(?i)(?:api_key|apikey|secret|token|password)[\s:=]+[\x22\x27]?([a-zA-Z0-9_\-\.]{16,})[\x22\x27]?").expect("Static Regex"), capture_group: 1 },
            SecretPattern { name: "DB Connection", regex: Regex::new(r"(?i)(?:postgres|mysql|redis|mongodb)://[a-zA-Z0-9_-]+:([a-zA-Z0-9_\-\.!@#$%^&*]+)@").expect("Static Regex"), capture_group: 1 },
        ]
    })
}

pub fn redact_text(text: &str) -> String {
    let mut redacted = text.to_string();
    let patterns = get_secret_patterns();

    for pattern in patterns {
        let mut new_redacted = String::new();
        let mut last_end = 0;

        for cap in pattern.regex.captures_iter(&redacted) {
            if let Some(m) = cap.get(pattern.capture_group) {
                // Check entropy if it's a generic secret regex
                if pattern.capture_group != 0 && shannon_entropy(m.as_str().as_bytes()) < 3.5 {
                    continue;
                }

                new_redacted.push_str(&redacted[last_end..m.start()]);
                new_redacted.push_str("********");
                last_end = m.end();
            }
        }
        
        if last_end > 0 {
            new_redacted.push_str(&redacted[last_end..]);
            redacted = new_redacted;
        }
    }

    redacted
}
