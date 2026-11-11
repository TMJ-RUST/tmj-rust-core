# TMJ Rust Core (`tmj-rust-core`)

Shared core library crate untuk ekosistem microservice **TMJ Rust** (`tmj-rust-auth`, `tmj-rust-users`, `tmj-rust-notification`, dan `tmj-rust-parameters`).

> Crate ini berupa **library (`src/lib.rs`)**, bukan binary service terpisah. Saat service konsumen di-build (baik di lokal maupun di Render), Cargo menarik crate ini dari GitHub dan meng-compile-nya secara statis ke dalam binary masing-masing service.

Dokumentasi teknis lengkap tersedia di [docs/index.md](docs/index.md) dan [AI_CONTEXT.md](AI_CONTEXT.md).

---

## Modul yang Disediakan

| Modul | Path | Deskripsi |
| --- | --- | --- |
| `config` | `src/config/` | `init_db(database_url, &Migrator)` (PostgreSQL pool + migrasi), `init_redis(redis_url)` (`ConnectionManager`), dan wrapper `JwtAccessSecret` |
| `dto` | `src/dto/` | Envelope respons standar `ApiResponse<T>`, `PaginatedData<T>`, `PaginationMeta`, dan `MessageResponse` |
| `entity` | `src/entity/` | Enum `UserRole` (`user`, `admin`) untuk SQLx, Serde, dan Utoipa OpenAPI |
| `error` | `src/error.rs` | Enum `AppError` terpusat beserta konversi `From<sqlx::Error>`, `From<jsonwebtoken::errors::Error>`, dan `IntoResponse` |
| `middleware` | `src/middleware/` | Extractor Axum `AuthenticatedUser` (verifikasi Bearer JWT + sesi aktif Redis) dan `RequestId` (`x-request-id` / `x-correlation-id`) |
| `repository` | `src/repository/` | `SessionRepository` untuk operasi sesi tunggal per user di Redis (`session:user:{id}`) dan health check `PING` |
| `parameters` | `src/parameters.rs` | `ParameterCache` untuk hash Redis code→value dan katalog pesan error, dengan fallback eksplisit tanpa cache lokal |
| `utils` | `src/utils/` | Utilitas password Argon2 (`hash_password`, `verify_password`) dan JWT (`Claims`, `create_access_token`, `create_refresh_token`, `verify_token`) |

---

## Cara Menggunakan di Service Lain (`Cargo.toml`)

```toml
[dependencies]
tmj-rust-core = { git = "https://github.com/meij0hn/tmj-rust-core.git", branch = "main" }
```

Saat ada pembaruan di `tmj-rust-core` yang sudah di-push ke `main`, jalankan perintah berikut pada masing-masing service untuk memperbarui `Cargo.lock`:

```bash
cargo update -p tmj-rust-core
```

Untuk rollout katalog terbaru, seluruh binary konsumen membawa histori migrasi identik hingga 0009 dan SQL schema eksplisit. Hentikan binary lama sebelum schema move; jalankan parameters dan pastikan publikasi startup berhasil sebelum konsumen lain. Module parameter hanya metadata, sehingga lookup Redis tetap code→value. Detail: [panduan parameter](docs/modules/parameters.md) dan [cutover service parameters](../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover). Perubahan dokumentasi saja tidak memerlukan pembaruan lockfile konsumen.
