"""HTTP fetching with error handling."""

import requests
from requests.exceptions import ConnectionError, RequestException, Timeout

from url_ingestion.models import ErrorType, FetchError, FetchResult, FetchSuccess, NormalizedURL

USER_AGENT = "RecipeScraper/1.0"
TIMEOUT_SECONDS = 30
MAX_REDIRECTS = 5
MAX_SIZE_BYTES = 10 * 1024 * 1024  # 10MB
CHUNK_SIZE = 8192


def fetch(normalized_url: NormalizedURL) -> FetchResult:
    """Fetch HTML content from a normalized URL.

    Args:
        normalized_url: The normalized URL to fetch.

    Returns:
        FetchSuccess with HTML content, or FetchError with error details.
    """
    url = normalized_url.url

    try:
        response = requests.get(
            url,
            timeout=TIMEOUT_SECONDS,
            headers={"User-Agent": USER_AGENT},
            allow_redirects=True,
            stream=True,
        )
        response.max_redirects = MAX_REDIRECTS
    except Timeout:
        return FetchError(
            error_type=ErrorType.NETWORK,
            message="Request timed out",
            url=url,
            details={"timeout_seconds": TIMEOUT_SECONDS},
        )
    except ConnectionError as e:
        return _handle_connection_error(e, url)
    except RequestException as e:
        return FetchError(
            error_type=ErrorType.NETWORK,
            message=f"Network error: {e}",
            url=url,
            details=None,
        )

    if not 200 <= response.status_code < 300:
        return _handle_http_error(response, url)

    content_type = response.headers.get("Content-Type", "")
    if not content_type.lower().startswith("text/html"):
        return FetchError(
            error_type=ErrorType.CONTENT_TYPE,
            message=f"Expected HTML but received {content_type or 'unknown content type'}",
            url=url,
            details={"content_type": content_type},
        )

    content_result = _read_content_with_limit(response, url)
    if isinstance(content_result, FetchError):
        return content_result

    final_url = None
    if response.url != url:
        final_url = response.url

    return FetchSuccess(
        url=normalized_url,
        html=content_result,
        status_code=response.status_code,
        content_type=content_type,
        final_url=final_url,
    )


def _handle_connection_error(e: ConnectionError, url: str) -> FetchError:
    """Convert connection errors to FetchError with appropriate message."""
    error_str = str(e).lower()

    if "name or service not known" in error_str or "nodename nor servname" in error_str:
        message = "DNS resolution failed: could not resolve hostname"
    elif "connection refused" in error_str:
        message = "Connection refused: server is not accepting connections"
    elif "no route to host" in error_str:
        message = "Network unreachable: no route to host"
    else:
        message = f"Connection failed: {e}"

    return FetchError(
        error_type=ErrorType.NETWORK,
        message=message,
        url=url,
        details=None,
    )


def _handle_http_error(response: requests.Response, url: str) -> FetchError:
    """Convert HTTP error status codes to FetchError."""
    status_code = response.status_code

    if 400 <= status_code < 500:
        if status_code == 404:
            message = "Page not found (404)"
        elif status_code == 403:
            message = "Access forbidden (403)"
        elif status_code == 401:
            message = "Authentication required (401)"
        else:
            message = f"Client error: HTTP {status_code}"
    else:
        if status_code == 500:
            message = "Internal server error (500)"
        elif status_code == 502:
            message = "Bad gateway (502)"
        elif status_code == 503:
            message = "Service unavailable (503)"
        else:
            message = f"Server error: HTTP {status_code}"

    return FetchError(
        error_type=ErrorType.HTTP,
        message=message,
        url=url,
        details={"status_code": status_code},
    )


def _read_content_with_limit(response: requests.Response, url: str) -> str | FetchError:
    """Read response content with size limit.

    Args:
        response: The HTTP response to read.
        url: The URL for error reporting.

    Returns:
        The content as a string, or FetchError if size limit exceeded.
    """
    content_bytes = b""

    for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
        content_bytes += chunk
        if len(content_bytes) > MAX_SIZE_BYTES:
            response.close()
            return FetchError(
                error_type=ErrorType.SIZE,
                message=f"Response exceeds {MAX_SIZE_BYTES // (1024 * 1024)}MB limit",
                url=url,
                details={"max_size_bytes": MAX_SIZE_BYTES},
            )

    encoding = response.encoding or "utf-8"
    try:
        return content_bytes.decode(encoding, errors="replace")
    except (LookupError, UnicodeDecodeError):
        return content_bytes.decode("utf-8", errors="replace")
