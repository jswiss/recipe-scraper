"""Data models for URL ingestion results."""

from dataclasses import dataclass
from enum import Enum


class ErrorType(Enum):
    """Categories of errors that can occur during URL ingestion."""

    VALIDATION = "validation"
    NETWORK = "network"
    HTTP = "http"
    CONTENT_TYPE = "content_type"
    SIZE = "size"


@dataclass(frozen=True)
class NormalizedURL:
    """A validated and normalized URL ready for fetching.

    Attributes:
        scheme: Protocol (http or https), lowercase.
        host: Domain in ASCII (Punycode if IDN).
        port: Port number, or None if default (80/443).
        path: URL path starting with "/".
        query: Query string without leading "?", or None.
        fragment: Fragment without leading "#", or None.
    """

    scheme: str
    host: str
    port: int | None
    path: str
    query: str | None
    fragment: str | None

    @property
    def url(self) -> str:
        """Reconstruct the full URL string."""
        result = f"{self.scheme}://{self.host}"
        if self.port is not None:
            result += f":{self.port}"
        result += self.path
        if self.query:
            result += f"?{self.query}"
        if self.fragment:
            result += f"#{self.fragment}"
        return result


@dataclass(frozen=True)
class FetchSuccess:
    """Successful fetch containing HTML content.

    Attributes:
        url: The normalized URL that was fetched.
        html: The HTML content of the page.
        status_code: HTTP status code (200-299).
        content_type: Full Content-Type header value.
        final_url: Final URL if redirected, or None.
    """

    url: NormalizedURL
    html: str
    status_code: int
    content_type: str
    final_url: str | None


@dataclass(frozen=True)
class FetchError:
    """Structured error information for failed operations.

    Attributes:
        error_type: Category of error.
        message: Human-readable error description.
        url: The URL that was attempted.
        details: Additional context (status code, etc.), or None.
    """

    error_type: ErrorType
    message: str
    url: str
    details: dict | None


# Type alias for the result of an ingestion operation
FetchResult = FetchSuccess | FetchError
