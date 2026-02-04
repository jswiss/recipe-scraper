"""URL Ingestion module for recipe scraping.

This module provides functionality to validate, normalize, and fetch URLs.
It returns structured results (success with HTML or failure with typed errors).

Basic usage:
    >>> from url_ingestion import ingest_url, FetchSuccess, FetchError
    >>> result = ingest_url("https://example.com/recipe")
    >>> if isinstance(result, FetchSuccess):
    ...     print(f"Got {len(result.html)} bytes")
    ... else:
    ...     print(f"Error: {result.message}")
"""

from url_ingestion.models import (
    ErrorType,
    FetchError,
    FetchResult,
    FetchSuccess,
    NormalizedURL,
)

__all__ = [
    "ingest_url",
    "validate_url",
    "FetchSuccess",
    "FetchError",
    "FetchResult",
    "ErrorType",
    "NormalizedURL",
]


def ingest_url(url: str) -> FetchResult:
    """Validate, normalize, and fetch a URL.

    Args:
        url: The URL to ingest.

    Returns:
        FetchSuccess with HTML content, or FetchError with error details.
    """
    from url_ingestion.fetcher import fetch
    from url_ingestion.normalizer import normalize
    from url_ingestion.validator import validate

    validation_result = validate(url)
    if isinstance(validation_result, FetchError):
        return validation_result

    normalized = normalize(validation_result)

    return fetch(normalized)


def validate_url(url: str) -> NormalizedURL | FetchError:
    """Validate and normalize a URL without fetching.

    Args:
        url: The URL to validate.

    Returns:
        NormalizedURL if valid, or FetchError with validation error details.
    """
    from url_ingestion.normalizer import normalize
    from url_ingestion.validator import validate

    validation_result = validate(url)
    if isinstance(validation_result, FetchError):
        return validation_result

    return normalize(validation_result)
