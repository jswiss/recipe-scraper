# Quickstart: URL Ingestion

**Feature**: 001-url-ingestion
**Date**: 2026-02-04

## Prerequisites

- Python 3.11+
- pip (Python package manager)

## Installation

```bash
# From repository root
pip install -e .

# Or install dependencies directly
pip install requests
```

## Basic Usage

### Ingest a URL

```python
from url_ingestion import ingest_url, FetchSuccess, FetchError

result = ingest_url("https://www.allrecipes.com/recipe/10813/best-chocolate-chip-cookies/")

if isinstance(result, FetchSuccess):
    print(f"Fetched {len(result.html)} bytes of HTML")
    print(f"Final URL: {result.final_url or result.url.url}")
else:
    print(f"Error [{result.error_type.value}]: {result.message}")
```

### Validate Without Fetching

```python
from url_ingestion import validate_url, NormalizedURL, FetchError

result = validate_url("HTTPS://Example.COM/recipe/")

if isinstance(result, NormalizedURL):
    print(f"Normalized: {result.url}")
    # Output: https://example.com/recipe
else:
    print(f"Invalid: {result.message}")
```

## Error Handling

The module returns typed errors that can be matched by category:

```python
from url_ingestion import ingest_url, FetchError, ErrorType

result = ingest_url(user_input)

if isinstance(result, FetchError):
    match result.error_type:
        case ErrorType.VALIDATION:
            print("Invalid URL format. Please check the URL and try again.")
        case ErrorType.NETWORK:
            print("Could not connect. Check your internet connection.")
        case ErrorType.HTTP:
            status = result.details.get("status_code", "unknown")
            print(f"Server returned error: HTTP {status}")
        case ErrorType.CONTENT_TYPE:
            print("URL does not point to an HTML page.")
        case ErrorType.SIZE:
            print("Response too large (>10MB). This doesn't look like a recipe page.")
```

## Examples

### Valid Recipe URL

```python
>>> result = ingest_url("https://www.seriouseats.com/best-chocolate-chip-cookies-recipe")
>>> isinstance(result, FetchSuccess)
True
>>> result.status_code
200
>>> "<!DOCTYPE html>" in result.html
True
```

### Invalid URL

```python
>>> result = ingest_url("not-a-url")
>>> result.error_type
ErrorType.VALIDATION
>>> result.message
"Invalid URL: missing scheme (http:// or https://)"
```

### Non-HTML Response

```python
>>> result = ingest_url("https://example.com/recipe.pdf")
>>> result.error_type
ErrorType.CONTENT_TYPE
>>> result.message
"Expected HTML but received application/pdf"
```

### Network Error

```python
>>> result = ingest_url("https://this-domain-does-not-exist.invalid/recipe")
>>> result.error_type
ErrorType.NETWORK
>>> "DNS" in result.message or "resolve" in result.message
True
```

## Configuration

The module uses sensible defaults aligned with the specification:

| Setting | Default | Description |
|---------|---------|-------------|
| Timeout | 30 seconds | Maximum time for fetch operation |
| Max redirects | 5 | Maximum redirect hops to follow |
| Max size | 10 MB | Maximum response body size |
| User-Agent | `RecipeScraper/1.0` | Identifies the client to servers |

These are not currently configurable - they are hardcoded per specification requirements.

## Testing Your Setup

Run this to verify the module is working:

```bash
python -c "
from url_ingestion import ingest_url, FetchSuccess
result = ingest_url('https://httpbin.org/html')
assert isinstance(result, FetchSuccess), f'Expected success, got {result}'
print('URL Ingestion module working correctly')
"
```

## Next Steps

After fetching HTML, you'll typically:

1. Pass `result.html` to a recipe parser module
2. Extract structured recipe data (ingredients, instructions, etc.)
3. Store the parsed recipe locally

See the recipe parsing feature (when implemented) for the next step in the pipeline.
