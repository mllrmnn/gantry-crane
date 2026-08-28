use serde::Serialize;

pub trait Entity<T: Serialize = Self>: Serialize {
    fn topic(&self, base_topic: &str, node_id: &str) -> String;
}

pub fn normalize_object_id(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for c in value.trim_matches('/').chars() {
        if c.is_ascii_alphanumeric() {
            normalized.push(c.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    let normalized = normalized.trim_matches('_').to_owned();
    if normalized.is_empty() {
        "unknown".into()
    } else {
        normalized
    }
}

pub fn normalize_unique_id_part(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for c in value.trim_matches('/').chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            normalized.push(c.to_ascii_lowercase());
            last_was_separator = false;
        } else if c == '_' || c.is_ascii_whitespace() {
            if !last_was_separator {
                normalized.push('_');
                last_was_separator = true;
            }
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    let normalized = normalized.trim_matches('_').to_owned();
    if normalized.is_empty() {
        "unknown".into()
    } else {
        normalized
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum DeviceClass {
    Restart,
    Update,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum EntityCategory {
    Config,
    Diagnostic,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum StateClass {
    Measurement,
    Total,
    TotalIncreasing,
}

#[cfg(test)]
mod test {
    use super::{normalize_object_id, normalize_unique_id_part};

    #[test]
    fn test_normalize_object_id() {
        assert_eq!(
            normalize_object_id("docker01-rrnuc / Docker Proxy"),
            "docker01_rrnuc_docker_proxy"
        );
        assert_eq!(normalize_object_id("///"), "unknown");
    }

    #[test]
    fn test_normalize_unique_id_part() {
        assert_eq!(normalize_unique_id_part("/Docker Proxy"), "docker_proxy");
        assert_eq!(normalize_unique_id_part("docker01-rrnuc"), "docker01-rrnuc");
        assert_eq!(normalize_unique_id_part("///"), "unknown");
    }
}
