# Feature Specification: URL Ingestion

**Feature Branch**: `001-url-ingestion`
**Created**: 2026-02-04
**Status**: Draft
**Input**: User description: "Accept a URL input, normalize and validate it, produce a fetch artifact indicating success or failure"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fetch Valid Recipe URL (Priority: P1)

A user provides a valid recipe website URL and receives the HTML content of that page for
further processing by downstream recipe parsing components.

**Why this priority**: This is the core happy path. Without successful URL fetching, no other
recipe scraping functionality can work. This delivers the fundamental value of the feature.

**Independent Test**: Can be fully tested by providing a known valid URL and verifying HTML
content is returned. Delivers the ability to retrieve web content for recipe extraction.

**Acceptance Scenarios**:

1. **Given** a valid, well-formed recipe URL (e.g., `https://example.com/recipe/123`),
   **When** the user submits the URL for ingestion,
   **Then** the system returns the HTML content of the page.

2. **Given** a valid URL with query parameters (e.g., `https://example.com/recipe?id=123`),
   **When** the user submits the URL for ingestion,
   **Then** the system preserves query parameters and returns the HTML content.

3. **Given** a valid URL with a trailing slash,
   **When** the user submits the URL for ingestion,
   **Then** the system normalizes the URL and returns the HTML content.

---

### User Story 2 - Handle Invalid URLs (Priority: P2)

A user provides a malformed or invalid URL and receives a clear, structured error explaining
why the URL could not be processed.

**Why this priority**: Error handling is essential for usability. Users need clear feedback
when something goes wrong so they can correct their input.

**Independent Test**: Can be fully tested by providing various invalid URL formats and
verifying appropriate error messages are returned.

**Acceptance Scenarios**:

1. **Given** a string that is not a valid URL (e.g., `not-a-url`, `ftp://wrong-protocol.com`),
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating the URL is invalid.

2. **Given** a URL missing the protocol (e.g., `example.com/recipe`),
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating the protocol is missing.

3. **Given** an empty string or whitespace-only input,
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating no URL was provided.

---

### User Story 3 - Handle Unreachable URLs (Priority: P3)

A user provides a valid URL format that cannot be fetched (network error, server down, 404,
etc.) and receives a structured error with details about the failure.

**Why this priority**: Network failures are common in web scraping. Users need to distinguish
between "bad URL format" and "URL is valid but unreachable" to troubleshoot effectively.

**Independent Test**: Can be fully tested by providing URLs to non-existent domains or pages
and verifying appropriate error responses.

**Acceptance Scenarios**:

1. **Given** a valid URL to a non-existent domain,
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating the domain could not be resolved.

2. **Given** a valid URL that returns a 404 status,
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating the page was not found.

3. **Given** a valid URL where the server times out,
   **When** the user submits it for ingestion,
   **Then** the system returns a structured error indicating a timeout occurred.

---

### Edge Cases

- IDN URLs are accepted and converted to Punycode during normalization
- URLs requiring authentication (401/403) return HTTP error per FR-009; auth is out of scope per Assumptions
- Responses exceeding 10MB are rejected with a size limit error
- Redirects (301, 302, 307) are followed up to 5 hops per FR-006
- Non-HTML responses (PDF, image, etc.) are rejected based on Content-Type header

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept a URL as text input
- **FR-002**: System MUST validate that the input conforms to URL syntax (RFC 3986)
- **FR-003**: System MUST only accept HTTP and HTTPS protocols
- **FR-004**: System MUST normalize URLs by:
  - Converting scheme and host to lowercase
  - Removing default ports (80 for HTTP, 443 for HTTPS)
  - Removing trailing slashes from paths (except root path)
  - Decoding unnecessarily percent-encoded characters
  - Converting internationalized domain names (IDN) to Punycode (RFC 3492)
- **FR-005**: System MUST produce a fetch artifact containing either:
  - Success: The HTML content of the fetched page
  - Failure: A structured error with error type and human-readable message
- **FR-006**: System MUST follow HTTP redirects (up to 5 redirects maximum)
- **FR-007**: System MUST set a reasonable timeout for fetch operations (30 seconds)
- **FR-008**: System MUST include a standard User-Agent header in requests
- **FR-009**: System MUST handle common HTTP error status codes (4xx, 5xx) with appropriate
  error messages
- **FR-010**: System MUST reject responses exceeding 10MB with a structured error indicating
  the response was too large
- **FR-011**: System MUST verify the Content-Type header indicates HTML (text/html) and reject
  non-HTML responses with a structured error

### Key Entities

- **URL Input**: The raw text string provided by the user representing the target web address
- **Normalized URL**: The validated and standardized form of the URL after processing
- **Fetch Artifact**: The result of the ingestion operation, containing either HTML content
  on success or error details on failure
- **Fetch Error**: Structured error information including error type (validation, network,
  HTTP status) and descriptive message

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Valid URLs return HTML content within 30 seconds for 95% of requests
- **SC-002**: Invalid URL formats are rejected with clear error messages within 100ms
- **SC-003**: 100% of fetch operations produce either HTML content or a structured error
  (no silent failures)
- **SC-004**: Users can distinguish between validation errors, network errors, and HTTP
  errors from the error response alone
- **SC-005**: URL normalization produces consistent output for equivalent URLs (e.g.,
  `HTTP://EXAMPLE.COM/` and `http://example.com` produce the same normalized form)

## Clarifications

### Session 2026-02-04

- Q: What is the maximum response size limit? → A: 10MB limit
- Q: How should non-HTML responses be handled? → A: Reject with error (check Content-Type)
- Q: How should internationalized domain names (IDN) be handled? → A: Accept and convert to Punycode

## Assumptions

- URLs are expected to point to publicly accessible web pages (no authentication required
  for MVP)
- The primary use case is fetching recipe pages, which are typically standard HTML documents
- Response size limits (if needed) can be determined during implementation based on typical
  recipe page sizes
- Rate limiting and politeness (crawl delays) are out of scope for this feature but may be
  added later
