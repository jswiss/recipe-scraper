"""URL validation using stdlib urllib.parse."""

from urllib.parse import ParseResult, urlparse

from url_ingestion.models import ErrorType, FetchError

ALLOWED_SCHEMES = {"http", "https"}


def validate(url: str) -> ParseResult | FetchError:
    """Validate URL syntax and protocol.

    Args:
        url: The raw URL string to validate.

    Returns:
        ParseResult if valid, or FetchError with validation details.
    """
    if not url or not url.strip():
        return FetchError(
            error_type=ErrorType.VALIDATION,
            message="No URL provided",
            url=url or "",
            details=None,
        )

    url = url.strip()

    try:
        parsed = urlparse(url)
    except Exception as e:
        return FetchError(
            error_type=ErrorType.VALIDATION,
            message=f"Invalid URL: {e}",
            url=url,
            details=None,
        )

    if not parsed.scheme:
        return FetchError(
            error_type=ErrorType.VALIDATION,
            message="Invalid URL: missing scheme (http:// or https://)",
            url=url,
            details=None,
        )

    if parsed.scheme.lower() not in ALLOWED_SCHEMES:
        return FetchError(
            error_type=ErrorType.VALIDATION,
            message=f"Invalid URL: scheme '{parsed.scheme}' not allowed (use http or https)",
            url=url,
            details=None,
        )

    if not parsed.netloc:
        return FetchError(
            error_type=ErrorType.VALIDATION,
            message="Invalid URL: missing host",
            url=url,
            details=None,
        )

    return parsed
