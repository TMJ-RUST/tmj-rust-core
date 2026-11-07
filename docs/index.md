# Dokumentasi TMJ Rust Core (`tmj-rust-core`)

Dokumentasi implementasi library bersama `tmj-rust-core`, diperiksa pada 5 November 2026. Entry point ringkas untuk agen: [AI_CONTEXT.md](../AI_CONTEXT.md).

| Topik | Dokumen | Sumber utama |
| --- | --- | --- |
| Peta modul & arsitektur | [Module architecture](architecture/module-architecture.md) | `src/lib.rs`, seluruh submodul |
| JWT, Argon2, extractor & sesi Redis | [Auth & session](modules/auth-and-session.md) | `src/utils/`, `src/middleware/auth.rs`, `src/repository/session.rs`, `src/entity/role.rs` |
| Envelope HTTP, error, request ID & logging REST | [HTTP & error](modules/http-and-error.md) | `src/dto/response.rs`, `src/error.rs`, `src/middleware/request_id.rs`, `src/middleware/http_logging.rs` |
| Koneksi DB & Redis | [Infrastructure](modules/infrastructure.md) | `src/config/mod.rs`, `src/config/database.rs`, `src/config/redis.rs` |

## Peran dalam Ekosistem TMJ

`tmj-rust-core` menyatukan komponen lintas-service agar `tmj-rust-auth` dan `tmj-rust-users` tidak menduplikasi kode infrastruktur, validasi sesi Redis, pembuatan/verifikasi JWT, hashing password, envelope respons, maupun pemetaan error HTTP.

Crate ini ditarik melalui Git (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`) dan dikompilasi langsung ke dalam binary service pemanggil.
