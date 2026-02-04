/**
 * Tauri Command Contracts: URL Ingestion
 *
 * TypeScript type definitions for Tauri commands exposed by the Rust backend.
 * These types define the contract between frontend and backend.
 *
 * Usage:
 *   import { invoke } from '@tauri-apps/api/core';
 *   const result = await invoke<FetchResult>('ingest_url', { url });
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Error type categories matching Rust ErrorType enum
 */
export type ErrorType =
  | 'validation'
  | 'network'
  | 'http'
  | 'content_type'
  | 'size';

/**
 * Normalized URL components
 */
export interface NormalizedUrl {
  scheme: 'http' | 'https';
  host: string;
  port: number | null;
  path: string;
  query: string | null;
  fragment: string | null;
}

/**
 * Successful fetch result
 */
export interface FetchSuccess {
  url: NormalizedUrl;
  html: string;
  status_code: number;
  content_type: string;
  final_url: string | null;
}

/**
 * Base error structure (all variants share these fields)
 */
interface FetchErrorBase {
  error_type: ErrorType;
  message: string;
  url: string;
}

/**
 * Validation error (invalid URL syntax or protocol)
 */
export interface ValidationError extends FetchErrorBase {
  error_type: 'validation';
}

/**
 * Network error (DNS, timeout, connection)
 */
export interface NetworkError extends FetchErrorBase {
  error_type: 'network';
  details?: Record<string, unknown>;
}

/**
 * HTTP error (4xx/5xx responses)
 */
export interface HttpError extends FetchErrorBase {
  error_type: 'http';
  status_code: number;
}

/**
 * Content type error (non-HTML response)
 */
export interface ContentTypeError extends FetchErrorBase {
  error_type: 'content_type';
  content_type: string;
}

/**
 * Size error (response too large)
 */
export interface SizeError extends FetchErrorBase {
  error_type: 'size';
  max_bytes: number;
}

/**
 * Union of all error types
 */
export type FetchError =
  | ValidationError
  | NetworkError
  | HttpError
  | ContentTypeError
  | SizeError;

/**
 * Result of URL ingestion (success or error)
 */
export type FetchResult = FetchSuccess | FetchError;

// ============================================================================
// Tauri Command Signatures
// ============================================================================

/**
 * Validate, normalize, and fetch a URL.
 *
 * @param url - The URL to ingest
 * @returns Promise resolving to FetchSuccess, or rejecting with FetchError
 *
 * @example
 * try {
 *   const result = await invoke<FetchSuccess>('ingest_url', { url: 'https://example.com/recipe' });
 *   console.log(result.html);
 * } catch (error) {
 *   const fetchError = error as FetchError;
 *   console.error(fetchError.message);
 * }
 */
export type IngestUrlCommand = (args: { url: string }) => Promise<FetchSuccess>;

/**
 * Validate and normalize a URL without fetching.
 *
 * @param url - The URL to validate
 * @returns Promise resolving to NormalizedUrl, or rejecting with FetchError (validation only)
 *
 * @example
 * try {
 *   const normalized = await invoke<NormalizedUrl>('validate_url', { url: 'https://EXAMPLE.COM/' });
 *   console.log(normalized.host); // 'example.com'
 * } catch (error) {
 *   const fetchError = error as ValidationError;
 *   console.error(fetchError.message);
 * }
 */
export type ValidateUrlCommand = (args: { url: string }) => Promise<NormalizedUrl>;

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Check if a result is a FetchSuccess
 */
export function isSuccess(result: FetchResult): result is FetchSuccess {
  return 'html' in result && 'status_code' in result;
}

/**
 * Check if a result is a FetchError
 */
export function isError(result: FetchResult): result is FetchError {
  return 'error_type' in result;
}

/**
 * Check error type
 */
export function isValidationError(error: FetchError): error is ValidationError {
  return error.error_type === 'validation';
}

export function isNetworkError(error: FetchError): error is NetworkError {
  return error.error_type === 'network';
}

export function isHttpError(error: FetchError): error is HttpError {
  return error.error_type === 'http';
}

export function isContentTypeError(error: FetchError): error is ContentTypeError {
  return error.error_type === 'content_type';
}

export function isSizeError(error: FetchError): error is SizeError {
  return error.error_type === 'size';
}
