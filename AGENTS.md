# Panduan agen TMJ Core

## Alur baca

1. Baca [AI_CONTEXT.md](AI_CONTEXT.md), lalu pilih halaman melalui [docs/index.md](docs/index.md) sebelum menelusuri source, menjawab pertanyaan implementasi, atau merencanakan perubahan. Tidak perlu membaca semua halaman.
2. Gunakan [skill docs-first](.agents/skills/tmj-core-docs-first/SKILL.md) untuk pemeriksaan dampak. Untuk integrasi, baca `AGENTS.md`, konteks, dan topik terkait hanya pada konsumen yang terdampak.
3. Tautan lintas repo mengasumsikan checkout saudara. Jika tidak tersedia, laporkan konteks yang belum dibaca; jangan menyimpulkan kontrak konsumen dari core saja. Panduan repo ini tetap menjadi pintu masuk saat core dibuka sendiri.

## Aturan perubahan

- Cocokkan dokumentasi dengan source aktual. Periksa dampak simbol publik, `FromRef`, JWT, sesi Redis, dan serialisasi terhadap auth/users/notification/parameters; laporkan perbedaan dan hasil verifikasi tanpa mengarang bukti runtime.
- Perbarui dokumen utama topik bersama perubahan perilaku/kontrak; kepemilikannya dipetakan di indeks. `AGENTS.md` menyimpan aturan kerja, `AI_CONTEXT.md` ringkasan/invariant, dan indeks navigasi. Gunakan tautan untuk detail milik topik/repo lain.
- Pertahankan deklarasi dependency Git `https://github.com/meij0hn/tmj-rust-core.git`, branch `main`, pada konsumen; jangan menggantinya dengan dependency path permanen.
- Penerbitan perubahan core ke Git `main` dilakukan jika diminta. Setelah revisi source/API tersedia, jalankan `cargo update -p tmj-rust-core` dan `cargo check` pada konsumen sesuai rollout. Perubahan dokumentasi saja tidak memerlukan pembaruan lockfile.
- Perubahan katalog atau schema wajib mengikuti [cutover parameters](../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover). Histori migrasi konsumen hingga `0009` harus identik; jangan mengubah migrasi yang sudah diterapkan. Core tidak membawa migrasi atau deployment service tersendiri.
- Jika perubahan core mengubah endpoint atau kontrak HTTP konsumen (method, path/query, header/auth, body, envelope/DTO, status code, atau saved example), perbarui YAML terkait di [collection](../tmj-rust-api-collection/). Repo `TMJ-RUST/tmj-rust-api-collection`, entry point `opencollection.yml`, adalah satu-satunya sumber; JSON Postman hanya export.
- Pertahankan perubahan pengguna. Jangan memasukkan nilai `.env`, secret, atau kredensial nyata ke kode maupun dokumentasi.
