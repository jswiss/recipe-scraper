# Tauri Command Contracts: Recipe Extraction

**Feature**: 003-recipe-extraction
**Date**: 2026-02-04

This document defines the Tauri IPC command contracts for recipe extraction.

---

## Commands Overview

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `extract_recipe` | HTML string | ExtractedRecipe \| ExtractionError | Primary extraction command |
| `extract_recipe_from_url` | URL string | ExtractedRecipe \| ExtractionError | Fetch + extract convenience |
| `check_ai_model_status` | None | ModelStatus | Check if AI model is available |
| `download_ai_model` | None | DownloadProgress stream | Download AI model for fallback |

---

## Command: `extract_recipe`

Extracts recipe data from HTML content using the priority chain: JSON-LD → Microdata → AI Fallback.

### Signature

```typescript
// Frontend invocation
const result = await invoke<ExtractedRecipe>('extract_recipe', { html: string });
```

### Rust Definition

```rust
#[tauri::command]
pub async fn extract_recipe(
    html: String,
    model_state: State<'_, Option<LlamaModel>>,
) -> Result<ExtractedRecipe, ExtractionError>
```

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `html` | String | Yes | Raw HTML content from URL ingestion |

### Response: Success

Returns `ExtractedRecipe` (see data-model.md for full schema).

```json
{
  "title": { "status": "found", "value": "Chocolate Cookies" },
  "description": { "status": "found", "value": "..." },
  "ingredients": [...],
  "instructions": [...],
  "prep_time_minutes": { "status": "found", "value": 15 },
  "cook_time_minutes": { "status": "found", "value": 10 },
  "servings": { "status": "found", "value": "24 cookies" },
  "images": { "status": "found", "value": ["https://..."] },
  "nutrition": { "status": "not_found", "justification": "Not provided" },
  "source": "json_ld"
}
```

### Response: Error

Returns `ExtractionError` with tagged union format.

```json
{
  "error_type": "no_recipe_found",
  "message": "Page does not contain recipe content",
  "html_preview": "<html>..."
}
```

### Error Types

| error_type | Description | When |
|------------|-------------|------|
| `no_recipe_found` | No recipe content detected | Page is not a recipe |
| `invalid_json_ld` | JSON-LD parsing failed | Malformed structured data |
| `invalid_microdata` | Microdata parsing failed | Malformed markup |
| `ai_extraction_failed` | AI model inference failed | Model error |
| `model_not_available` | AI model not downloaded | First use without model |

---

## Command: `extract_recipe_from_url`

Convenience command that combines URL ingestion and recipe extraction.

### Signature

```typescript
const result = await invoke<ExtractedRecipe>('extract_recipe_from_url', { url: string });
```

### Rust Definition

```rust
#[tauri::command]
pub async fn extract_recipe_from_url(
    url: String,
    client: State<'_, HttpClient>,
    model_state: State<'_, Option<LlamaModel>>,
) -> Result<ExtractedRecipe, ExtractFromUrlError>
```

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | String | Yes | URL of recipe page to fetch and extract |

### Response: Success

Same as `extract_recipe` - returns `ExtractedRecipe`.

### Response: Error

Extended error type that includes fetch errors.

```rust
#[derive(Error, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ExtractFromUrlError {
    /// URL fetch failed (from url_ingestion module)
    Fetch(FetchError),
    /// Recipe extraction failed
    Extraction(ExtractionError),
}
```

---

## Command: `check_ai_model_status`

Checks if the local AI model is downloaded and ready for fallback extraction.

### Signature

```typescript
const status = await invoke<ModelStatus>('check_ai_model_status');
```

### Rust Definition

```rust
#[tauri::command]
pub fn check_ai_model_status(
    model_state: State<'_, Option<LlamaModel>>,
) -> ModelStatus
```

### Response

```rust
#[derive(Serialize, Deserialize)]
pub struct ModelStatus {
    /// Whether model is downloaded
    pub downloaded: bool,
    /// Whether model is loaded in memory
    pub loaded: bool,
    /// Model file path (if downloaded)
    pub model_path: Option<String>,
    /// Model size in bytes (if downloaded)
    pub model_size_bytes: Option<u64>,
    /// Model name/version
    pub model_name: String,
}
```

```json
{
  "downloaded": true,
  "loaded": true,
  "model_path": "/Users/josh/.config/recipe-scraper/models/gemma-2-2b-q4.gguf",
  "model_size_bytes": 1800000000,
  "model_name": "gemma-2-2b-it-q4_k_m"
}
```

---

## Command: `download_ai_model`

Downloads the AI model for fallback extraction. This is a long-running operation that emits progress events.

### Signature

```typescript
// Start download
await invoke('download_ai_model');

// Listen for progress
await listen<DownloadProgress>('ai-model-download-progress', (event) => {
  console.log(`${event.payload.downloaded_bytes} / ${event.payload.total_bytes}`);
});
```

### Rust Definition

```rust
#[tauri::command]
pub async fn download_ai_model(
    app: AppHandle,
    model_state: State<'_, Arc<Mutex<Option<LlamaModel>>>>,
) -> Result<(), ModelDownloadError>
```

### Progress Event

Emits `ai-model-download-progress` events:

```rust
#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Total bytes to download
    pub total_bytes: u64,
    /// Download speed in bytes/second
    pub speed_bytes_per_sec: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
}
```

### Response: Success

Returns `()` on successful download. Model is automatically loaded.

### Response: Error

```rust
#[derive(Error, Serialize)]
pub enum ModelDownloadError {
    #[error("Network error: {message}")]
    Network { message: String },
    #[error("Disk error: {message}")]
    Disk { message: String },
    #[error("Verification failed: {message}")]
    Verification { message: String },
}
```

---

## Integration with URL Ingestion

The recipe extraction module integrates with the existing URL ingestion module:

```
┌─────────────┐      ┌─────────────────┐      ┌───────────────────┐
│   Frontend  │ ──── │  ingest_url     │ ──── │  extract_recipe   │
│             │      │  (returns HTML) │      │  (returns Recipe) │
└─────────────┘      └─────────────────┘      └───────────────────┘

OR (convenience command):

┌─────────────┐      ┌───────────────────────┐
│   Frontend  │ ──── │  extract_recipe_from_ │
│             │      │  url (fetch+extract)  │
└─────────────┘      └───────────────────────┘
```

### Example Frontend Usage

```typescript
// Option 1: Two-step (more control)
const { html } = await invoke<FetchSuccess>('ingest_url', { url: recipeUrl });
const recipe = await invoke<ExtractedRecipe>('extract_recipe', { html });

// Option 2: One-step (convenience)
const recipe = await invoke<ExtractedRecipe>('extract_recipe_from_url', { url: recipeUrl });
```

---

## State Management

The recipe extraction module requires managed state for the AI model:

```rust
// In lib.rs
pub fn run() {
    let http_client = create_http_client().expect("Failed to create HTTP client");
    let model_state: Arc<Mutex<Option<LlamaModel>>> = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .manage(http_client)
        .manage(model_state)
        .invoke_handler(tauri::generate_handler![
            // URL ingestion
            ingest_url,
            validate_url,
            // Recipe extraction
            extract_recipe,
            extract_recipe_from_url,
            check_ai_model_status,
            download_ai_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
