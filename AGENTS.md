# Panduan agen TMJ Core (`tmj-rust-core`)

Sebelum menelusuri source, merencanakan perubahan, atau mengubah `tmj-rust-core`, baca [AI_CONTEXT.md](AI_CONTEXT.md), [indeks dokumentasi](docs/index.md), lalu dokumen modul yang relevan. Gunakan [skill docs-first](.agents/skills/tmj-core-docs-first/SKILL.md).

## Saat mengubah source

- `tmj-rust-core` dipakai bersama oleh auth/users/notification/parameters melalui Git dependency (`https://github.com/meij0hn/tmj-rust-core.git`, branch `main`).
- Periksa dampak perubahan simbol, trait bound (`FromRef`), klaim JWT, key Redis, dan format serialisasi `ApiResponse`/`AppError` terhadap keempat service konsumen.
- Setelah mengubah source/API core, perbarui dokumentasi di `docs/` dan `AI_CONTEXT.md`, push perubahan ke branch `main` (jika diminta), lalu jalankan `cargo update -p tmj-rust-core` dan `cargo check` pada auth/users/notification/parameters sesuai rollout. Perubahan dokumentasi saja tidak memerlukan pembaruan lockfile.
- Untuk integrasi katalog, ikuti [panduan parameters](docs/modules/parameters.md): histori migrasi konsumen identik hingga 0009, SQL schema eksplisit, stop/drain binary lama sebelum schema move, dan startup publication parameters sukses sebelum rollout konsumen. Redis tetap code→value; module bukan namespace Redis.
- Perbarui YAML request dan saved response service konsumen yang terdampak di `../tmj-rust-api-collection/` jika perubahan core mengubah kontrak HTTP. Repo collection adalah satu-satunya sumber; JSON Postman hanya export sesuai kebutuhan, bukan salinan manual.
- Jangan memasukkan nilai `.env`, secret, atau kredensial nyata ke dalam kode maupun dokumentasi.
