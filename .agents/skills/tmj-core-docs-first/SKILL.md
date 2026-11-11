---
name: tmj-core-docs-first
description: Baca dokumentasi TMJ Rust Core terlebih dahulu saat menjawab pertanyaan, merencanakan, meninjau, atau mengubah library tmj-rust-core beserta integrasinya di auth, users, notification, dan parameters.
---

# TMJ Core: dokumentasi terlebih dahulu

Aturan kerja dan kewajiban pembaruan ada di [AGENTS.md](../../../AGENTS.md). Gunakan path relatif terhadap root checkout core.

1. Baca [konteks ringkas](../../../AI_CONTEXT.md), lalu pilih halaman melalui [indeks](../../../docs/index.md) sesuai tugas. Jangan membaca seluruh dokumentasi untuk perubahan satu modul.
2. Setelah memahami kontrak, verifikasi simbol publik dan pemanggil aktual. Tentukan service terdampak di antara auth/users/notification/parameters; baca konteks integrasinya. Perhatikan parameter Redis dan katalog error selain kontrak JWT/sesi/HTTP.
3. Cocokkan perubahan dengan dokumentasi modul dan collection konsumen sesuai aturan repo. Pisahkan fakta source, interpretasi, dan hasil pengujian yang benar-benar dijalankan; laporkan konteks yang hilang atau tidak konsisten.
