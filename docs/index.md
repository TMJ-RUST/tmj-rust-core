# Dokumentasi TMJ Rust Core

Aturan kerja: [AGENTS.md](../AGENTS.md). Ringkasan library: [AI_CONTEXT.md](../AI_CONTEXT.md). Pilih halaman sesuai tugas; tabel berikut juga menetapkan dokumen utama tiap topik. Halaman ringkasan cukup menautkan detail, bukan menyalinnya.

| Tugas | Dokumen yang dibaca |
| --- | --- |
| Memahami struktur, dependency, dan simbol publik | [Module architecture](architecture/module-architecture.md) |
| Mengubah JWT, password, extractor, atau sesi Redis | [Auth & session](modules/auth-and-session.md) |
| Mengubah envelope, pagination, error, request ID, atau logging REST | [HTTP & error](modules/http-and-error.md) |
| Mengubah pembacaan parameter, fallback, atau katalog `messageError` | [Parameters](modules/parameters.md) |
| Mengubah koneksi PostgreSQL/Redis atau integrasi migrator | [Infrastructure](modules/infrastructure.md) |
| Merencanakan rollout katalog/schema lintas konsumen | [Kontrak parameter](modules/parameters.md), lalu [cutover parameters](../../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover) |

## Batas dokumentasi lintas repo

- Core memiliki kontrak library: JWT/sesi, envelope/error/logging, koneksi, dan pembacaan parameter. Flow login/recovery dimiliki [auth](../../tmj-rust-auth/AI_CONTEXT.md); RBAC database dimiliki [users](../../tmj-rust-users/AI_CONTEXT.md).
- Schema katalog, publisher, bootstrap, dan koordinasi migrasi dimiliki [parameters](../../tmj-rust-parameters/docs/index.md). Core tidak menyalin prosedur tersebut.
- Untuk perubahan integrasi, lanjutkan ke `AGENTS.md` dan konteks konsumen yang terdampak, termasuk [notification](../../tmj-rust-notification/AI_CONTEXT.md). Kontrak endpoint tetap didokumentasikan pada service; request dan saved response dipelihara di [repo collection](../../tmj-rust-api-collection/).

Tautan lintas repo membutuhkan checkout saudara. Jika tidak tersedia, nyatakan batas pemeriksaan. Dokumen kontrak menjelaskan implementasi; perintah verifikasi dan catatan pengujian terdahulu bukan bukti bahwa pemeriksaan telah dijalankan pada perubahan saat ini.
