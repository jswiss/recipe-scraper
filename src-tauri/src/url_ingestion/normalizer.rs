use url::Url;

use crate::url_ingestion::models::NormalizedUrl;

/// Normalizes a parsed URL into a NormalizedUrl struct.
///
/// Normalization rules:
/// 1. Scheme lowercased (already by url crate)
/// 2. Host lowercased and converted to Punycode if IDN
/// 3. Default ports removed (80 for http, 443 for https)
/// 4. Path must start with "/" (default to "/" if empty)
/// 5. Trailing slashes removed from path (except root "/")
pub fn normalize(url: &Url) -> NormalizedUrl {
    let scheme = url.scheme().to_lowercase();

    // Host normalization: lowercase and convert IDN to Punycode
    let host = normalize_host(url.host_str().unwrap_or(""));

    // Port normalization: remove default ports
    let port = normalize_port(url.port(), &scheme);

    // Path normalization: ensure leading slash, remove trailing slash
    let path = normalize_path(url.path());

    // Query and fragment (without leading ? and #)
    let query = url.query().map(String::from);
    let fragment = url.fragment().map(String::from);

    NormalizedUrl {
        scheme,
        host,
        port,
        path,
        query,
        fragment,
    }
}

/// Normalizes the host: lowercase and convert IDN to Punycode.
fn normalize_host(host: &str) -> String {
    let lowercased = host.to_lowercase();

    // Convert IDN to Punycode using the idna crate
    match idna::domain_to_ascii(&lowercased) {
        Ok(ascii) => ascii,
        Err(_) => lowercased, // Fall back to lowercased on error
    }
}

/// Removes default ports (80 for http, 443 for https).
fn normalize_port(port: Option<u16>, scheme: &str) -> Option<u16> {
    match port {
        Some(80) if scheme == "http" => None,
        Some(443) if scheme == "https" => None,
        p => p,
    }
}

/// Normalizes the path: ensures leading slash, removes trailing slash.
fn normalize_path(path: &str) -> String {
    // Ensure leading slash
    let with_leading = if path.is_empty() || !path.starts_with('/') {
        format!("/{}", path)
    } else {
        path.to_string()
    };

    // Remove trailing slash (except for root "/")
    if with_leading.len() > 1 && with_leading.ends_with('/') {
        with_leading[..with_leading.len() - 1].to_string()
    } else {
        with_leading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_simple_url() {
        let url = Url::parse("https://example.com/path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.scheme, "https");
        assert_eq!(normalized.host, "example.com");
        assert_eq!(normalized.port, None);
        assert_eq!(normalized.path, "/path");
        assert_eq!(normalized.query, None);
        assert_eq!(normalized.fragment, None);
    }

    #[test]
    fn test_normalize_uppercase_host() {
        let url = Url::parse("https://EXAMPLE.COM/Path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.host, "example.com");
        // Note: path case is preserved
        assert_eq!(normalized.path, "/Path");
    }

    #[test]
    fn test_normalize_default_https_port() {
        let url = Url::parse("https://example.com:443/path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.port, None);
    }

    #[test]
    fn test_normalize_default_http_port() {
        let url = Url::parse("http://example.com:80/path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.port, None);
    }

    #[test]
    fn test_normalize_non_default_port() {
        let url = Url::parse("https://example.com:8443/path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.port, Some(8443));
    }

    #[test]
    fn test_normalize_trailing_slash() {
        let url = Url::parse("https://example.com/path/").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.path, "/path");
    }

    #[test]
    fn test_normalize_root_path() {
        let url = Url::parse("https://example.com/").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.path, "/");
    }

    #[test]
    fn test_normalize_idn_domain() {
        // münchen.de should become xn--mnchen-3ya.de
        let url = Url::parse("https://münchen.de/").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.host, "xn--mnchen-3ya.de");
    }

    #[test]
    fn test_normalize_with_query_and_fragment() {
        let url = Url::parse("https://example.com/path?foo=bar#section").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.query, Some("foo=bar".to_string()));
        assert_eq!(normalized.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_to_url_string() {
        let url = Url::parse("https://example.com:8443/path?foo=bar#section").unwrap();
        let normalized = normalize(&url);

        assert_eq!(
            normalized.to_url_string(),
            "https://example.com:8443/path?foo=bar#section"
        );
    }

    #[test]
    fn test_to_url_string_without_port() {
        let url = Url::parse("https://example.com/path").unwrap();
        let normalized = normalize(&url);

        assert_eq!(normalized.to_url_string(), "https://example.com/path");
    }
}
