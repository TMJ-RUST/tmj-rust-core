# Modul HTTP Response, RequestId, dan Error Handling

## 1. Envelope Respons (`src/dto/response.rs`)
- `ApiResponse<T>`: Envelope standar dengan field `camelCase` (`statusCode`, `reffCode`, `message`, `messageError`, `data`).
- `PaginationMeta` & `PaginatedData<T>`: Struktur paginasi standar (`items`, `pagination: { page, limit, totalItems, totalPages }`).
- `MessageResponse`: DTO pesan status sederhana untuk dokumentasi skema OpenAPI.

## 2. Extractor `RequestId` (`src/middleware/request_id.rs`)
- Membaca header `x-request-id`, atau fallback ke `x-correlation-id`.
- Bila kedua header tidak ada atau kosong, otomatis membangkitkan `Uuid::new_v4()`.

## 3. `AppError` (`src/error.rs`)
Menyatukan seluruh varian error dari `tmj-rust-auth` dan `tmj-rust-users`:

| Varian `AppError` | HTTP Status | `statusCode` | `message` |
| --- | --- | --- | --- |
| `InvalidCredentials` | `401 Unauthorized` | `"01"` | `"Authentication Failed"` |
| `InvalidToken`, `TokenExpired`, `Unauthorized(_)` | `401 Unauthorized` | `"02"` | `"Unauthorized"` |
| `ValidationError(_)` (termasuk FK violation SQLx) | `400 Bad Request` | `"03"` | `"Validation Error"` |
| `NotFound(_)` (termasuk `RowNotFound` SQLx) | `404 Not Found` | `"04"` | `"Resource Not Found"` |
| `Forbidden(_)` | `403 Forbidden` | `"05"` | `"Access Forbidden"` |
| `UserAlreadyExists(_)`, `Conflict(_)` (termasuk unique violation SQLx) | `409 Conflict` | `"09"` | `"Resource Conflict"` |
| `DatabaseError(_)`, `InternalServerError(_)` | `500 Internal Server Error` | `"99"` | `"Internal Server Error"` |
