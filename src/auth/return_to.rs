use std::fmt;

use url::Url;

/// Maximum accepted UTF-8 byte length for a local post-login navigation target.
pub const MAX_SAFE_RETURN_TO_BYTES: usize = 2_048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeReturnTo(String);

impl SafeReturnTo {
    /// Parses one bounded, non-recursive, same-origin-relative navigation target.
    ///
    /// # Errors
    ///
    /// Returns an error when the target is external, malformed, ambiguous after percent decoding,
    /// recursive into login/authentication routes, or exceeds the configured byte bound.
    pub fn parse(input: &str) -> Result<Self, SafeReturnToError> {
        if input.is_empty() || input.len() > MAX_SAFE_RETURN_TO_BYTES {
            return Err(SafeReturnToError::Invalid);
        }
        if !input.starts_with('/') || input.starts_with("//") {
            return Err(SafeReturnToError::Invalid);
        }
        if input
            .chars()
            .any(|character| character == '\\' || character.is_control())
        {
            return Err(SafeReturnToError::Invalid);
        }

        let decoded = percent_decode(input)?;
        if decoded
            .chars()
            .any(|character| character == '\\' || character.is_control())
        {
            return Err(SafeReturnToError::Invalid);
        }

        let path_end = input.find(['?', '#']).unwrap_or(input.len());
        let raw_path = &input[..path_end];
        let decoded_path = percent_decode(raw_path)?;
        if decoded_path.contains('%') || contains_encoded_path_delimiter(raw_path) {
            return Err(SafeReturnToError::Invalid);
        }
        if decoded_path
            .split('/')
            .any(|component| component == "." || component == "..")
        {
            return Err(SafeReturnToError::Invalid);
        }
        if is_auth_path(&decoded_path) {
            return Err(SafeReturnToError::Invalid);
        }

        // Parsing against a constant authority verifies URL syntax without consulting a request
        // host or allowing a caller-provided authority to influence the target.
        Url::parse(&format!("https://flowl.invalid{input}"))
            .map_err(|_| SafeReturnToError::Invalid)?;

        Ok(Self(input.to_string()))
    }

    pub fn fallback(input: &str) -> Self {
        Self::parse(input).unwrap_or_else(|_| Self("/".to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SafeReturnTo {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SafeReturnTo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeReturnToError {
    Invalid,
}

impl fmt::Display for SafeReturnToError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("return target must be a safe local path")
    }
}

impl std::error::Error for SafeReturnToError {}

fn percent_decode(input: &str) -> Result<String, SafeReturnToError> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1).ok_or(SafeReturnToError::Invalid)?;
            let low = *bytes.get(index + 2).ok_or(SafeReturnToError::Invalid)?;
            decoded.push((hex_value(high)? << 4) | hex_value(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).map_err(|_| SafeReturnToError::Invalid)
}

const fn hex_value(value: u8) -> Result<u8, SafeReturnToError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(SafeReturnToError::Invalid),
    }
}

fn contains_encoded_path_delimiter(path: &str) -> bool {
    let bytes = path.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Ok(high) = bytes
                .get(index + 1)
                .copied()
                .ok_or(SafeReturnToError::Invalid)
                .and_then(hex_value)
            else {
                return true;
            };
            let Ok(low) = bytes
                .get(index + 2)
                .copied()
                .ok_or(SafeReturnToError::Invalid)
                .and_then(hex_value)
            else {
                return true;
            };
            if matches!((high << 4) | low, b'/' | b'\\' | b'?' | b'#') {
                return true;
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    false
}

fn is_auth_path(path: &str) -> bool {
    matches!(path, "/login" | "/auth") || path.starts_with("/login/") || path.starts_with("/auth/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_local_paths_queries_and_fragments() {
        for target in [
            "/",
            "/plants/42",
            "/plants/42?tab=care",
            "/plants/42?tab=care#entry-7",
            "/search?q=snake%20plant",
        ] {
            assert_eq!(SafeReturnTo::parse(target).unwrap().as_str(), target);
        }
    }

    #[test]
    fn rejects_external_and_protocol_relative_targets() {
        for target in [
            "https://attacker.example/",
            "http://attacker.example/",
            "//attacker.example/",
            "///attacker.example/",
            r"/\\attacker.example/",
        ] {
            assert!(SafeReturnTo::parse(target).is_err(), "{target}");
        }
    }

    #[test]
    fn rejects_controls_backslashes_and_malformed_escapes() {
        for target in [
            "/plants\\42",
            "/plants\n42",
            "/plants%00",
            "/plants%5c42",
            "/plants%",
            "/plants%2",
            "/plants%zz",
            "/plants%ff",
        ] {
            assert!(SafeReturnTo::parse(target).is_err(), "{target:?}");
        }
    }

    #[test]
    fn rejects_encoded_separators_and_normalization_confusion() {
        for target in [
            "/plants%2f42",
            "/plants%3fnext=1",
            "/plants%23section",
            "/./plants",
            "/plants/../login",
            "/%2e/login",
            "/plants/%2e%2e/login",
            "/plants/%252e%252e/login",
        ] {
            assert!(SafeReturnTo::parse(target).is_err(), "{target}");
        }
    }

    #[test]
    fn rejects_login_and_auth_routes_including_encoded_equivalents() {
        for target in [
            "/login",
            "/login/",
            "/login/reset",
            "/auth",
            "/auth/",
            "/auth/callback",
            "/log%69n",
            "/a%75th/callback",
        ] {
            assert!(SafeReturnTo::parse(target).is_err(), "{target}");
        }
    }

    #[test]
    fn rejects_oversized_targets() {
        let target = format!("/{}", "a".repeat(MAX_SAFE_RETURN_TO_BYTES));
        assert!(SafeReturnTo::parse(&target).is_err());
    }

    #[test]
    fn unsafe_values_fall_back_to_root() {
        assert_eq!(
            SafeReturnTo::fallback("https://attacker.example/").as_str(),
            "/"
        );
        assert_eq!(SafeReturnTo::fallback("/plants/42").as_str(), "/plants/42");
    }
}
