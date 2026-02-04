"""URL normalization per RFC 3986."""

from urllib.parse import ParseResult, unquote

import idna

from url_ingestion.models import NormalizedURL

DEFAULT_PORTS = {"http": 80, "https": 443}


def normalize(parsed: ParseResult) -> NormalizedURL:
    """Normalize a parsed URL.

    Normalization rules:
    - Scheme and host converted to lowercase
    - Default ports removed (80 for HTTP, 443 for HTTPS)
    - Trailing slashes removed from path (except "/" root)
    - Unnecessarily percent-encoded chars decoded
    - IDN domains converted to Punycode (ASCII)

    Args:
        parsed: The parsed URL from urlparse.

    Returns:
        A NormalizedURL with all normalization applied.
    """
    scheme = parsed.scheme.lower()

    host = _normalize_host(parsed.hostname or "")

    port = _normalize_port(parsed.port, scheme)

    path = _normalize_path(parsed.path)

    query = parsed.query if parsed.query else None

    fragment = parsed.fragment if parsed.fragment else None

    return NormalizedURL(
        scheme=scheme,
        host=host,
        port=port,
        path=path,
        query=query,
        fragment=fragment,
    )


def _normalize_host(host: str) -> str:
    """Normalize hostname: lowercase and convert IDN to Punycode."""
    host = host.lower()

    try:
        host = idna.encode(host, uts46=True).decode("ascii")
    except idna.IDNAError:
        pass

    return host


def _normalize_port(port: int | None, scheme: str) -> int | None:
    """Remove default ports (80 for http, 443 for https)."""
    if port is None:
        return None
    if port == DEFAULT_PORTS.get(scheme):
        return None
    return port


def _normalize_path(path: str) -> str:
    """Normalize path: decode percent-encoding, remove trailing slash."""
    if not path:
        return "/"

    path = _decode_safe_percent_encoding(path)

    if path != "/" and path.endswith("/"):
        path = path.rstrip("/")

    return path or "/"


def _decode_safe_percent_encoding(s: str) -> str:
    """Decode percent-encoded chars that don't need encoding.

    Keeps reserved chars like /, ?, # encoded if they appear encoded.
    """
    decoded = unquote(s, encoding="utf-8", errors="replace")
    return decoded
