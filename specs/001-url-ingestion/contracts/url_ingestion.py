"""
URL Ingestion Module Contract

This file defines the public API contract for the URL ingestion module.
It serves as the interface specification - implementation details may vary.

Feature: 001-url-ingestion
Date: 2026-02-04
"""

from dataclasses import dataclass
from enum import Enum
from typing import Union


class ErrorType(Enum):
    """Categories of errors that can occur during URL ingestion."""

    VALIDATION = "validation"    # URL syntax or protocol invalid
    NETWORK = "network"          # Connection failed (DNS, timeout, refused)
    HTTP = "http"                # Server returned error status (4xx, 5xx)
    CONTENT_TYPE = "content_type"  # Response is not HTML
    SIZE = "size"                # Response exceeds 10MB limit


@dataclass(frozen=True)
class NormalizedURL:
    """A validated and standardized URL ready for fetching."""

    scheme: str          # "http" or "https" (lowercase)
    host: str            # Domain in ASCII (Punycode if IDN)
    port: int | None     # Only set if non-default (not 80/443)
    path: str            # URL path starting with "/", no trailing slash
    query: str | None    # Query string without leading "?"
    fragment: str | None # Fragment without leading "#"

    @property
    def url(self) -> str:
        """Reconstruct the full normalized URL string."""
        result = f"{self.scheme}://{self.host}"
        if self.port:
            result += f":{self.port}"
        result += self.path
        if self.query:
            result += f"?{self.query}"
        if self.fragment:
            result += f"#{self.fragment}"
        return result


@dataclass(frozen=True)
class FetchSuccess:
    """Successful fetch result containing HTML content."""

    url: NormalizedURL      # The normalized URL that was fetched
    html: str               # The HTML content (≤10MB)
    status_code: int        # HTTP status code (200-299)
    content_type: str       # Full Content-Type header value
    final_url: str | None   # Final URL if redirected, else None


@dataclass(frozen=True)
class FetchError:
    """Structured error information for failed operations."""

    error_type: ErrorType   # Category of error
    message: str            # Human-readable error description
    url: str                # The URL that was attempted
    details: dict | None    # Additional context (status code, etc.)


# Union type for function return
FetchResult = Union[FetchSuccess, FetchError]


def ingest_url(raw_url: str) -> FetchResult:
    """
    Ingest a URL: validate, normalize, fetch, and return result.

    This is the main entry point for URL ingestion. It performs:
    1. Validation: Checks URL syntax and protocol (HTTP/HTTPS only)
    2. Normalization: Standardizes URL format per RFC 3986
    3. Fetching: Retrieves HTML content with redirect following
    4. Verification: Checks Content-Type and size limits

    Args:
        raw_url: The URL string to ingest (as provided by user)

    Returns:
        FetchSuccess: If the URL was successfully fetched
        FetchError: If any step failed, with categorized error type

    Behavior:
        - Follows up to 5 redirects (301, 302, 307, 308)
        - Times out after 30 seconds
        - Rejects responses >10MB
        - Rejects non-HTML Content-Type
        - Converts IDN domains to Punycode

    Example:
        >>> result = ingest_url("https://example.com/recipe")
        >>> if isinstance(result, FetchSuccess):
        ...     print(result.html[:100])
        >>> else:
        ...     print(f"Error: {result.error_type.value}: {result.message}")
    """
    raise NotImplementedError("Contract only - see implementation")


def validate_url(raw_url: str) -> NormalizedURL | FetchError:
    """
    Validate and normalize a URL without fetching.

    Useful for pre-validation before batch processing.

    Args:
        raw_url: The URL string to validate

    Returns:
        NormalizedURL: If valid, the normalized URL
        FetchError: If invalid, with error_type=VALIDATION
    """
    raise NotImplementedError("Contract only - see implementation")
