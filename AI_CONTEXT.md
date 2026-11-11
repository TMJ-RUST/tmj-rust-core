# AI Context — TMJ Rust Core

Konteks implementasi berdasarkan inspeksi statis; bukan laporan pengujian runtime. Pilih bacaan lanjutan melalui [indeks dokumentasi](docs/index.md); aturan kerja ada di [AGENTS.md](AGENTS.md).

## Peran dan batas tanggung jawab

Library Rust bersama untuk auth/users/notification/parameters: JWT/password, extractor autentikasi, sesi Redis, respons/error HTTP, logging, koneksi database, dan pembacaan katalog parameter. Entry point: [src/lib.rs](src/lib.rs); dependency: [Cargo.toml](Cargo.toml).

Keempat service menarik crate melalui Git `https://github.com/meij0hn/tmj-rust-core.git`, branch `main`, lalu menautkannya secara statis. Core tidak di-deploy sebagai service dan tidak memiliki migrasi sendiri; binary konsumen memasok migrator.

## Invariant utama

- Autentikasi membutuhkan access JWT dan sesi Redis aktif yang cocok. Kontrak `Claims`, `SessionRepository`, dan `FromRef` dipakai bersama seluruh konsumen; lihat [auth dan sesi](docs/modules/auth-and-session.md).
- Envelope dan pagination memakai `camelCase`; `statusCode` bisnis berbeda dari status HTTP. Katalog error hanya mengganti `messageError` pada respons yang ditandai; lihat [HTTP/error](docs/modules/http-and-error.md) dan [parameter](docs/modules/parameters.md).
- `ParameterCache` membaca Redis langsung dengan fallback eksplisit, tanpa cache lokal atau retry. PostgreSQL dan publikasi dimiliki service parameters; kontrak reader serta batas integrasinya ada di [parameter](docs/modules/parameters.md).
- Migrasi dan rollout dimiliki konsumen, bukan core. Prosedur lintas service berada di [operasi parameters](../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover).
- Logger HTTP hanya menyamarkan nama field yang mengandung `password`; token/OTP dan nilai lain dapat tercatat. Batasi akses serta retensi log; detail ada di [logging REST](docs/modules/http-and-error.md#4-logging-rest-srcmiddlewarehttp_loggingrs).
