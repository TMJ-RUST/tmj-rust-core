---
name: tmj-core-docs-first
description: Baca dokumentasi TMJ Rust Core terlebih dahulu saat menjawab pertanyaan, merencanakan, meninjau, atau mengubah library tmj-rust-core beserta integrasinya di tmj-rust-auth dan tmj-rust-users.
---

# TMJ Core: dokumentasi terlebih dahulu

Folder dokumentasi lokal: `/Users/meijohn/Desktop/DEV/tmj-rust/tmj-rust-core/docs`.
Jika checkout berpindah, gunakan `docs/` relatif terhadap root `tmj-rust-core`.

Sebelum menelusuri source, menyusun rencana teknis, atau mengubah kode:

1. Baca `AI_CONTEXT.md` dan `docs/index.md`.
2. Baca dokumen arsitektur dan modul terkait di `docs/`:
   - `architecture/module-architecture.md` untuk pemetaan modul dan simbol publik.
   - `modules/auth-and-session.md` untuk JWT, Argon2, `AuthenticatedUser`, dan `SessionRepository`.
   - `modules/http-and-error.md` untuk `ApiResponse`, `PaginatedData`, `RequestId`, dan `AppError`.
   - `modules/infrastructure.md` untuk `init_db` dan `init_redis`.
3. Verifikasi source aktual di `src/` serta efeknya pada `tmj-rust-auth` dan `tmj-rust-users`.
4. Perbarui dokumen di `docs/` dan `AI_CONTEXT.md` setiap kali kontrak, tipe publik, atau perilaku berubah.
5. Selalu tulis ulang (rewrite/update) `/Users/meijohn/Desktop/DEV/tmj-rust/TMJ-Service.postman_collection.json` (`../TMJ-Service.postman_collection.json` dari root `tmj-rust-core`) jika perubahan pada `tmj-rust-core` memengaruhi kontrak request atau response HTTP (seperti `ApiResponse`, `PaginatedData`, `AppError`, header/token autentikasi) pada `tmj-auth-service` maupun `tmj-users-service`.
