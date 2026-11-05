# AI Context — TMJ Rust Core

Status: inspeksi statis working tree, 5 November 2026. Mulai dengan [indeks dokumentasi](docs/index.md) sebelum membaca atau mengubah source `tmj-rust-core`.

## Overview dan stack

Library crate bersama (`src/lib.rs`) untuk microservice `tmj-rust-auth` dan `tmj-rust-users`. Rust 2021, Axum 0.8, SQLx 0.8 (PostgreSQL), Redis 0.27 (`ConnectionManager`), jsonwebtoken 9.3, Argon2 0.5, Serde, tracing, chrono, uuid, dan Utoipa 6.0.

Crate ini diintegrasikan ke service konsumen melalui dependency Git (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`) dan di-link secara statis saat kompilasi (tidak di-deploy sebagai container/service terpisah).

## Baca sesuai tugas

| Tugas | Dokumen |
| --- | --- |
| Peta modul dan simbol | [Module architecture](docs/architecture/module-architecture.md) |
| Mengubah JWT, password, extractor auth, atau sesi Redis | [Auth & session module](docs/modules/auth-and-session.md) |
| Mengubah `ApiResponse`, `AppError`, atau `RequestId` | [HTTP & error module](docs/modules/http-and-error.md) |
| Mengubah koneksi PostgreSQL / Redis | [Infrastructure module](docs/modules/infrastructure.md) |

## Invariants penting

1. **Kontrak Sesi Redis (`SessionRepository`)**: Key Redis memakai format `session:user:{user_id}` dengan hash fields `session_id`, `refresh_token`, `user_id`, `username`, `role` dan TTL sesuai masa berlaku refresh token. Baik `tmj-rust-auth` (saat login/refresh/logout) maupun `AuthenticatedUser` di kedua service bergantung pada struktur key ini. Sumber: [src/repository/session.rs](src/repository/session.rs), [src/middleware/auth.rs](src/middleware/auth.rs).
2. **Kontrak JWT (`Claims` & `AuthenticatedUser`)**: Access token wajib memiliki `token_type == "access"` dan `jti` yang sama dengan `session_id` aktif di Redis. `AuthenticatedUser` membutuhkan `JwtAccessSecret: FromRef<S>` dan `SessionRepository: FromRef<S>` pada `AppState` service pemanggil. Sumber: [src/utils/token.rs](src/utils/token.rs), [src/middleware/auth.rs](src/middleware/auth.rs).
3. **Envelope & Error (`ApiResponse` & `AppError`)**: Seluruh field `ApiResponse`, `PaginationMeta`, dan `PaginatedData` diserialisasi dalam `camelCase` (`statusCode`, `reffCode`, `messageError`, `totalItems`, `totalPages`). Kode status bisnis: `"00"` sukses, `"01"` gagal autentikasi, `"02"` unauthorized/token, `"03"` validasi, `"04"` not found, `"05"` forbidden, `"09"` conflict, `"99"` internal/DB error. Sumber: [src/dto/response.rs](src/dto/response.rs), [src/error.rs](src/error.rs).
4. **Dampak lintas-repo**: Setiap perubahan publik di `tmj-rust-core` wajib di-commit & push ke repo `tmj-rust-core`, lalu dijalankan `cargo update -p tmj-rust-core` pada `tmj-rust-auth` dan `tmj-rust-users`.
