# Dokumentasi TMJ Rust Core (`tmj-rust-core`)

Dokumentasi implementasi library bersama `tmj-rust-core`, diperiksa pada 5 November 2026. Entry point ringkas untuk agen: [AI_CONTEXT.md](../AI_CONTEXT.md).

| Topik | Dokumen | Sumber utama |
| --- | --- | --- |
| Peta modul & arsitektur | [Module architecture](architecture/module-architecture.md) | `src/lib.rs`, seluruh submodul |
| JWT, Argon2, extractor & sesi Redis | [Auth & session](modules/auth-and-session.md) | `src/utils/`, `src/middleware/auth.rs`, `src/repository/session.rs`, `src/entity/role.rs` |
| Envelope HTTP, error, request ID & logging REST | [HTTP & error](modules/http-and-error.md) | `src/dto/response.rs`, `src/error.rs`, `src/middleware/request_id.rs`, `src/middleware/http_logging.rs` |
| Parameter global & katalog messageError Redis | [Parameters](modules/parameters.md) | `src/parameters.rs`, `src/middleware/error_catalog.rs`, `src/error.rs` |
| Koneksi DB & Redis | [Infrastructure](modules/infrastructure.md) | `src/config/mod.rs`, `src/config/database.rs`, `src/config/redis.rs` |

## Peran dalam Ekosistem TMJ

`tmj-rust-core` menyatukan komponen lintas-service untuk `tmj-rust-auth`, `tmj-rust-users`, `tmj-rust-notification`, dan `tmj-rust-parameters`: infrastruktur, validasi sesi Redis, pembuatan/verifikasi JWT, hashing password, envelope respons, pemetaan error HTTP, dan pembacaan katalog Redis.

Crate ini ditarik melalui Git (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`) dan dikompilasi langsung ke dalam binary service pemanggil.

Panduan rollout parameter diperbarui 11 November 2026: startup parameters harus sukses memublikasikan katalog sebelum konsumen berjalan, seluruh binary membawa histori migrasi identik hingga 0009, dan perpindahan schema memerlukan penghentian binary lama. Redis tetap code→value walaupun parameter kini memakai module/code/value. Lihat [parameters](modules/parameters.md) dan [infrastructure](modules/infrastructure.md); pembaruan ini bukan laporan pengujian runtime.
