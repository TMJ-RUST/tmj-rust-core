# Modul HTTP Response, RequestId, dan Error Handling

## 1. Envelope Respons (`src/dto/response.rs`)
- `ApiResponse<T>`: Envelope standar dengan field `camelCase` (`statusCode`, `reffCode`, `message`, `messageError`, `data`).
- `PaginationMeta` & `PaginatedData<T>`: Struktur paginasi standar (`items`, `pagination: { page, limit, totalItems, totalPages }`).
- `MessageResponse`: DTO pesan status sederhana untuk dokumentasi skema OpenAPI.

`statusCode` adalah kode bisnis: `"00"` untuk sukses; kode kegagalan dijelaskan pada tabel `AppError` di bawah. Kode bisnis berbeda dari status HTTP. Field `ApiResponse`, `PaginationMeta`, dan `PaginatedData` memakai serialisasi `camelCase`.

## 2. Extractor `RequestId` (`src/middleware/request_id.rs`)
- Membaca header `x-request-id`, atau fallback ke `x-correlation-id`.
- Bila kedua header tidak ada atau kosong, otomatis membangkitkan `Uuid::new_v4()`.

## 3. `AppError` (`src/error.rs`)
Menyediakan varian error bersama untuk auth/users/notification/parameters:

| Varian `AppError` | HTTP Status | `statusCode` | `message` |
| --- | --- | --- | --- |
| `InvalidCredentials` | `401 Unauthorized` | `"01"` | `"Authentication Failed"` |
| `InvalidToken`, `TokenExpired`, `Unauthorized(_)` | `401 Unauthorized` | `"02"` | `"Unauthorized"` |
| `ValidationError(_)` (termasuk FK violation SQLx) | `400 Bad Request` | `"03"` | `"Validation Error"` |
| `NotFound(_)` (termasuk `RowNotFound` SQLx) | `404 Not Found` | `"04"` | `"Resource Not Found"` |
| `Forbidden(_)` | `403 Forbidden` | `"05"` | `"Access Forbidden"` |
| `UserAlreadyExists(_)`, `Conflict(_)` (termasuk unique violation SQLx) | `409 Conflict` | `"09"` | `"Resource Conflict"` |
| `DatabaseError(_)`, `InternalServerError(_)` | `500 Internal Server Error` | `"99"` | `"Internal Server Error"` |

`messageError` tidak lagi mengambil detail dinamis dari setiap varian error. Respons memakai mapping constant per `statusCode`, lalu middleware `error_catalog::resolve_error_message` dapat menggantinya dengan hasil HGET Redis pada setiap error. HTTP status, `statusCode`, `message`, referensi dan header tetap. Tanpa middleware, envelope tetap valid dengan constant default. Lihat [katalog error dan parameter](parameters.md) untuk pemasangan, fallback, batas waktu lookup dan API error recovery.

## 4. Logging REST (`src/middleware/http_logging.rs`)

`log_http` dipasang sebagai middleware Axum `from_fn` di auth, users, dan notification, di luar CORS agar preflight dan header respons CORS ikut tercatat. Logger mengirim satu event `INFO` bertarget `tmj_http` per pertukaran HTTP; aktifkan dengan `RUST_LOG=tmj_http=info` atau tambahkan directive tersebut ke filter service. `tmj_http=off` menonaktifkan pencatatan tanpa membaca body.

Format event: pemisah `========================================================================`, `INBOUND >>` (request ID, URI, method, headers), `OUTBOUND <<` (status HTTP numerik, headers, response body), lalu `-->Nms`. Satu event menjaga pasangan request/response tetap bersama ketika request berjalan bersamaan; prefix waktu/level dari tracing tetap ada.

- Request body tidak dibaca atau dicatat. Nilai query hanya disamarkan jika nama parameter mengandung `password` (tanpa peka kapital, nama percent-encoded didekode untuk pencocokan). Query lain, parameter tanpa `=`, dan path ditampilkan apa adanya.
- Semua nilai header dicatat, termasuk Authorization, cookie, API key, Upstash-Signature, dan header bertanda sensitive. Hanya header dengan nama yang mengandung `password` disamarkan; karakter kontrol di-escape agar struktur log tetap utuh.
- Hanya body JSON (`application/json` atau `application/*+json`) tanpa encoding non-identity yang diinspeksi. Maksimal 16 KiB payload disalin selama frame dikirim ke klien; frame data, trailer, status, header, dan error stream tidak diubah. Tidak ada buffering seluruh respons sebelum pengiriman.
- Redaksi JSON rekursif hanya mengganti nilai field yang namanya mengandung `password` tanpa peka kapital, termasuk `password`, `newPassword`, `new_password`, dan `password_hash`. Access/refresh/reset token, OTP, reference, identitas pengguna/penerima, variables, konten email, dan detail error tidak disamarkan. Ini kebijakan logging password-only, bukan pemindai rahasia pada teks bebas. Log dapat memuat kredensial aktif dan data pribadi: batasi akses serta retensinya; `tmj_http=off` menonaktifkan log REST.
- Body terlalu besar, non-JSON, streaming SSE/biner, encoded, JSON rusak, stream gagal, atau tidak selesai dikonsumsi memakai penanda omission; potongan mentah tidak pernah dicatat. Body kosong ditandai `[empty]`.
- Durasi JSON dihitung sampai frame terakhir/akhir stream; jika body dilewati sejak awal, durasi berhenti ketika header respons tersedia. Log bukan bukti seluruh byte sudah diterima klien. Saat stream terputus, log dicatat ketika body dilepas.
- Request ID log memakai `x-request-id`, lalu `x-correlation-id`, jika berisi 1–128 karakter ASCII alfanumerik/`-_.`; selain itu UUID baru. ID ini hanya untuk log, tidak mengganti extractor `RequestId`, `reffCode`, atau header API.

Pengujian perilaku: `cargo test --test http_logging` memeriksa bahwa hanya password yang disamarkan pada JSON bersarang/header/query (termasuk nama percent-encoded), sedangkan authorization/token/cookie/OTP/nilai lain terlihat. Pengujian juga mencakup batas 16 KiB, JSON UTF-8 lintas-frame, trailer, serta JSON rusak/stream error/tidak selesai. Smoke HTTP terisolasi membuktikan password header/query/JSON teredaksi sementara Authorization, API key, access/refresh token, OTP, dan cookie tercatat serta respons klien tidak berubah.
