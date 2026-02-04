# Rust Development Patterns for Recipe Scraper

## Error Handling

### Using thiserror for Custom Errors

```rust
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum FetchError {
    #[error("{message}")]
    Validation { message: String, url: String },

    #[error("{message}")]
    Network { message: String, url: String, details: Option<String> },

    #[error("{message}")]
    Http { message: String, url: String, status_code: u16 },
}
```

### Converting Between Error Types

```rust
// Map external errors to your error type
let response = client
    .get(&url)
    .send()
    .await
    .map_err(|e| FetchError::Network {
        message: format!("Request failed: {}", e),
        url: url.to_string(),
        details: Some(e.to_string()),
    })?;
```

## Tauri Command Patterns

### Basic Async Command

```rust
#[tauri::command]
pub async fn my_command(arg: String) -> Result<MyResponse, MyError> {
    // Command implementation
}
```

### Command with State

```rust
use tauri::State;

pub struct MyState(pub SomeType);

#[tauri::command]
pub async fn my_command(
    arg: String,
    state: State<'_, MyState>
) -> Result<MyResponse, MyError> {
    state.0.do_something(&arg).await
}
```

### Registering Commands

```rust
// In lib.rs or main.rs
tauri::Builder::default()
    .manage(MyState(SomeType::new()))
    .invoke_handler(tauri::generate_handler![
        my_command,
        another_command,
    ])
    .run(tauri::generate_context!())
```

## reqwest HTTP Client

### Client Configuration

```rust
use reqwest::Client;
use std::time::Duration;

pub fn create_client() -> Result<Client, Error> {
    Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(30))
        .user_agent("MyApp/1.0")
        .build()
        .map_err(|e| /* convert error */)
}
```

### Making Requests

```rust
let response = client
    .get(&url)
    .send()
    .await?;

// Check status
if response.status().is_client_error() {
    return Err(/* error */);
}

// Get content type
let content_type = response
    .headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

// Read body
let body = response.text().await?;
```

## URL Handling

### Parsing URLs

```rust
use url::Url;

let parsed = Url::parse(&input)?;
let scheme = parsed.scheme();        // "https"
let host = parsed.host_str();        // Some("example.com")
let port = parsed.port();            // None or Some(8080)
let path = parsed.path();            // "/path"
```

### IDN/Punycode Conversion

```rust
use idna;

let ascii_host = idna::domain_to_ascii(&unicode_host)?;
// "münchen.de" -> "xn--mnchen-3ya.de"
```

## Serde Serialization

### Tagged Enums for JSON

```rust
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Success { data: String },
    Error { message: String },
}

// Serializes to:
// { "type": "success", "data": "..." }
// { "type": "error", "message": "..." }
```

### Optional Fields

```rust
#[derive(Serialize)]
pub struct Response {
    pub required: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<String>,
}
```

## Testing Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let result = validate("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_case() {
        let result = validate("");
        assert!(matches!(result, Err(FetchError::Validation { .. })));
    }
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_fetch() {
    let client = create_client().unwrap();
    let result = fetch(&client, &url).await;
    assert!(result.is_ok());
}
```
