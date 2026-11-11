# AI Context — TMJ Rust Core

Status: inspeksi statis working tree, 5 November 2026; panduan konsumen/rollout parameter diperbarui 11 November 2026. Mulai dengan [indeks dokumentasi](docs/index.md) sebelum membaca atau mengubah source `tmj-rust-core`.

## Overview dan stack

Library crate bersama (`src/lib.rs`) untuk microservice `tmj-rust-auth`, `tmj-rust-users`, `tmj-rust-notification`, dan `tmj-rust-parameters`. Rust 2021, Axum 0.8, SQLx 0.8 (PostgreSQL), Redis 0.27 (`ConnectionManager`), jsonwebtoken 9.3, Argon2 0.5, Serde, tracing, chrono, uuid, dan Utoipa 6.0.

Crate ini diintegrasikan ke service konsumen melalui dependency Git (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`) dan di-link secara statis saat kompilasi (tidak di-deploy sebagai container/service terpisah).

## Baca sesuai tugas

| Tugas | Dokumen |
| --- | --- |
| Peta modul dan simbol | [Module architecture](docs/architecture/module-architecture.md) |
| Mengubah JWT, password, extractor auth, atau sesi Redis | [Auth & session module](docs/modules/auth-and-session.md) |
| Mengubah `ApiResponse`, `AppError`, `RequestId`, atau logging REST | [HTTP & error module](docs/modules/http-and-error.md) |
| Mengubah koneksi PostgreSQL / Redis | [Infrastructure module](docs/modules/infrastructure.md) |
| Membaca parameter global atau mengganti messageError per statusCode | [Parameter & error catalogue](docs/modules/parameters.md) |

## Invariants penting

1. **Kontrak Sesi Redis (`SessionRepository`)**: Key Redis memakai format `session:user:{user_id}` dengan hash fields `session_id`, `refresh_token`, `user_id`, `username`, `role` dan TTL sesuai masa berlaku refresh token. Baik `tmj-rust-auth` (saat login/refresh/logout) maupun `AuthenticatedUser` di service konsumen bergantung pada struktur key ini. Sumber: [src/repository/session.rs](src/repository/session.rs), [src/middleware/auth.rs](src/middleware/auth.rs).
2. **Kontrak JWT (`Claims` & `AuthenticatedUser`)**: Access token wajib memiliki `token_type == "access"` dan `jti` yang sama dengan `session_id` aktif di Redis. `AuthenticatedUser` membutuhkan `JwtAccessSecret: FromRef<S>` dan `SessionRepository: FromRef<S>` pada `AppState` service pemanggil. Sumber: [src/utils/token.rs](src/utils/token.rs), [src/middleware/auth.rs](src/middleware/auth.rs).
3. **Envelope & Error (`ApiResponse` & `AppError`)**: Seluruh field `ApiResponse`, `PaginationMeta`, dan `PaginatedData` diserialisasi dalam `camelCase` (`statusCode`, `reffCode`, `messageError`, `totalItems`, `totalPages`). Kode status bisnis: `"00"` sukses, `"01"` gagal autentikasi, `"02"` unauthorized/token, `"03"` validasi, `"04"` not found, `"05"` forbidden, `"09"` conflict, `"99"` internal/DB error. Sumber: [src/dto/response.rs](src/dto/response.rs), [src/error.rs](src/error.rs).
   Logging REST bersama: `middleware::http_logging::log_http` untuk middleware Axum `from_fn`, satu event multiline `tmj_http=info` per request/response. Hanya field bernama password (termasuk newPassword/password_hash, tanpa peka kapital) pada header/query/JSON yang disamarkan; authorization/token/OTP/nilai lain dicatat. Capture JSON maksimal 16 KiB, tanpa membaca request body atau menahan seluruh response stream. Batasi akses/retensi log; detail ada di dokumen HTTP.
4. **Dampak lintas-repo**: Perubahan API/source publik di `tmj-rust-core` perlu diterbitkan ke repo core, lalu diperbarui melalui `cargo update -p tmj-rust-core` pada auth/users/notification/parameters sesuai rollout. Pembaruan dokumentasi saja tidak memerlukan perubahan lockfile konsumen.
5. **Parameter dan katalog error**: `parameters::ParameterCache` membaca hash Redis global `tmj:parameters:system` dan `tmj:parameters:errors`, tanpa cache lokal/TTL/retry. Lookup maksimal 250 ms; numeric default eksplisit ada di consumer, fallback error berupa mapping constant per statusCode di core. `middleware::error_catalog::resolve_error_message` mengoverride hanya messageError pada respons bertanda `AppError`/`error::catalog_error_response`, sebelum logger. Error standalone tetap envelope valid melalui body lazy satu kali serialisasi. Integrasi mencakup auth/users/notification dan service parameters; detail di [parameters](docs/modules/parameters.md).
6. **Rollout katalog dan schema**: Parameters otomatis memublikasikan katalog setelah migrasi dan sebelum HTTP bind; startup gagal jika publikasi gagal. Wajibkan startup sukses sebelum rollout konsumen, refresh manual hanya pemulihan. Histori migrasi semua binary harus identik hingga 0009: 0008 memakai module/code/value, 0009 memindahkan tabel domain ke schema eksplisit. Module hanya metadata; Redis tetap code→value dan core tidak membaca tabel katalog. Stop/drain seluruh binary lama sebelum schema move, siapkan USAGE untuk role runtime terpisah, dan ikuti [cutover parameters](../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover). Core tidak membawa migrasi atau service deployment tersendiri.
