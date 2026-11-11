# Module architecture (`tmj-rust-core`)

Stack konteks: Rust 2021, Axum 0.8, SQLx 0.8 (PostgreSQL), Redis 0.27 (`ConnectionManager`), jsonwebtoken 9.3, Argon2 0.5, Serde, tracing, chrono, uuid, dan Utoipa 6.0. Requirement dependency ada di [Cargo.toml](../../Cargo.toml); ini bukan laporan versi deployment. Dokumentasi awal berasal dari inspeksi statis 5 November 2026, dengan panduan parameter/rollout diperbarui 11 November 2026, bukan bukti pengujian runtime.

| Modul / Konsep | Simbol Publik | Source |
| --- | --- | --- |
| Crate root | `config`, `dto`, `entity`, `error`, `middleware`, `parameters`, `repository`, `utils` | [src/lib.rs](../../src/lib.rs) |
| Config & koneksi | `init_db`, `init_redis`, `JwtAccessSecret` | [config/mod.rs](../../src/config/mod.rs), [database.rs](../../src/config/database.rs), [redis.rs](../../src/config/redis.rs) |
| Entity bersama | `UserRole` (`User`, `Admin`) | [entity/role.rs](../../src/entity/role.rs) |
| DTO bersama | `ApiResponse<T>`, `PaginatedData<T>`, `PaginationMeta`, `MessageResponse` | [dto/response.rs](../../src/dto/response.rs) |
| Error terpusat | `AppError` + `IntoResponse` | [error.rs](../../src/error.rs) |
| Middleware / extractor | `AuthenticatedUser`, `RequestId`, `X_REQUEST_ID`, `X_CORRELATION_ID` | [middleware/auth.rs](../../src/middleware/auth.rs), [middleware/request_id.rs](../../src/middleware/request_id.rs) |
| Logging REST | `http_logging::log_http` (middleware Axum `from_fn`, event `tmj_http`) | [middleware/http_logging.rs](../../src/middleware/http_logging.rs) |
| Parameter & katalog error | `ParameterCache`, `NumericParameter`, `keys`, `DEFAULT_ERROR_MESSAGES`, `default_error_message`, `error_catalog::resolve_error_message`, `error::catalog_error_response` | [parameters.rs](../../src/parameters.rs), [middleware/error_catalog.rs](../../src/middleware/error_catalog.rs), [error.rs](../../src/error.rs) |
| Repository bersama | `SessionRepository` | [repository/session.rs](../../src/repository/session.rs) |
| Utilitas kriptografi & token | `hash_password`, `verify_password`, `Claims`, `create_access_token`, `create_refresh_token`, `verify_token` | [utils/password.rs](../../src/utils/password.rs), [utils/token.rs](../../src/utils/token.rs) |
