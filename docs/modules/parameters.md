# Parameter Redis dan katalog error

`parameters::ParameterCache` adalah client read-only untuk cache global milik service `tmj-rust-parameters`. PostgreSQL menjadi sumber utama service pengelola; consumer tidak mengakses tabel parameter atau memanggil HTTP service parameters pada jalur request.

## Namespace dan nilai

| Redis hash | Field | Value |
| --- | --- | --- |
| `tmj:parameters:system` | `parameter_key` global | String |
| `tmj:parameters:errors` | `status_code`, termasuk nol di depan | String `messageError` |

Cache tidak memiliki TTL. CRUD service pengelola mencoba publikasi setelah commit DB; refresh manual memulihkan publikasi yang gagal. Tidak ada retry worker/outbox. Namespace sesi dan state recovery tidak dibersihkan saat refresh katalog.

## Client consumer

Buat `ParameterCache::new(connection_manager.clone())` sebelum connection manager dipindahkan ke `SessionRepository`. Satu clone berbagi manager Redis, bukan membuat snapshot parameter lokal.

- `string(key, default).await`: HGET langsung; key hilang atau lookup gagal memakai default eksplisit caller. String kosong tetap string yang valid.
- `numbers(&[NumericParameter; N]).await`: satu HMGET untuk kelompok terkait. Setiap `NumericParameter` berisi `key`, `default`, `min`, dan `max`; nilai tidak ada, parsing i64 gagal, atau di luar rentang memakai default field tersebut. Default angka berada di consumer.
- `message_error(status_code).await`: HGET setiap error. Key hilang, pesan kosong, atau lookup gagal memakai `DEFAULT_ERROR_MESSAGES`; kode tidak dikenal memakai pesan generik.

Lookup dibatasi 250 ms termasuk penantian manager reconnect. Tidak ada retry pada client ini dan tidak ada nilai stale di memori. Warning fallback memakai target tracing `tmj_parameters`; tambahkan `tmj_parameters=warn` ke RUST_LOG bila diperlukan untuk diagnosis. Jangan menaruh kredensial di katalog; logger HTTP project hanya menyamarkan field password.

`parameters::keys` mendefinisikan nama key uppercase untuk pagination, durasi JWT, durasi recovery, dan kuota recovery. Nilai tetap string; parsing dan default operasional menjadi tanggung jawab consumer.

## Error response async

Pasang `middleware::error_catalog::resolve_error_message` menggunakan `from_fn_with_state(parameters.clone(), resolve_error_message)`, di dalam `log_http` dan di luar CORS. Middleware hanya menangani respons yang ditandai oleh `AppError` atau `error::catalog_error_response`; response sukses serta rejection JSON/query/routing bawaan Axum tidak berubah.

`IntoResponse` tetap sinkron. Error menyimpan envelope lazy sampai body dipoll; middleware menyelesaikan lookup async sebelum body dikirim. JSON diserialisasi satu kali, tanpa decode/re-encode respons. Tanpa middleware, body tetap menghasilkan envelope dengan constant fallback yang valid. State payload request-local, tidak ada global cache atau state task-local; mutex tidak ditahan lintas await.

Katalog hanya mengganti `messageError`. HTTP status, `statusCode`, `message`, `reffCode`, header request ID, dan data null dipertahankan. Recovery dapat menggunakan `catalog_error_response(http_status, status_code, message)` lalu menambahkan Retry-After sesuai kontraknya.

Mapping fallback per kode:

| statusCode | messageError default |
| --- | --- |
| `01` | Invalid username/email or password |
| `02` | Invalid or expired authentication token |
| `03` | Invalid request data |
| `04` | Resource not found |
| `05` | Access forbidden |
| `09` | Resource conflict |
| `99` | An internal error occurred |
| Tidak dikenal | An error occurred |

## Verifikasi

`cargo test` mencakup envelope lazy/default standalone dan log HTTP. Pengujian Redis nyata bersifat opt-in dan harus memakai database Redis disposable:

```bash
TMJ_PARAMETER_TEST_REDIS_URL=redis://127.0.0.1:16389/11 cargo test --test error_catalog -- --include-ignored
```

Test tersebut menghapus kedua hash katalog pada database test yang diberikan; jangan memakai Redis produksi. Smoke HTTP terisolasi juga memeriksa perubahan nilai live, parsing invalid, key hilang, serta preservasi HTTP/statusCode/message/request ID.
