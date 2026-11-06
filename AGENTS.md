# Panduan agen TMJ Core (`tmj-rust-core`)

Sebelum menelusuri source, merencanakan perubahan, atau mengubah `tmj-rust-core`, baca [AI_CONTEXT.md](AI_CONTEXT.md), [indeks dokumentasi](docs/index.md), lalu dokumen modul yang relevan. Gunakan [skill docs-first](.agents/skills/tmj-core-docs-first/SKILL.md).

## Saat mengubah source

- `tmj-rust-core` dipakai bersama oleh `tmj-rust-auth` dan `tmj-rust-users` melalui Git dependency (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`).
- Periksa dampak perubahan simbol, trait bound (`FromRef`), klaim JWT, key Redis, dan format serialisasi `ApiResponse`/`AppError` terhadap kedua service konsumen.
- Setelah mengubah `tmj-rust-core`, perbarui dokumentasi di `docs/` dan `AI_CONTEXT.md`, push perubahan ke branch `main` (jika diminta), lalu jalankan `cargo update -p tmj-rust-core` dan `cargo check` pada `tmj-rust-auth` serta `tmj-rust-users`.
- Selalu tulis ulang (rewrite/update) `../TMJ-Service.postman_collection.json` jika perubahan di `tmj-rust-core` mengubah struktur request atau response HTTP pada service konsumen.
- Jangan memasukkan nilai `.env`, secret, atau kredensial nyata ke dalam kode maupun dokumentasi.
