# Parameter Redis dan katalog error

`parameters::ParameterCache` adalah client read-only untuk cache global milik service `tmj-rust-parameters`. PostgreSQL menjadi sumber utama service pengelola; consumer tidak mengakses tabel parameter atau memanggil HTTP service parameters pada jalur request.

## Namespace dan nilai

| Redis hash | Field | Value |
| --- | --- | --- |
| `tmj:parameters:system` | `code` unik global | String `value` |
| `tmj:parameters:errors` | `status_code`, termasuk nol di depan | String `messageError` |

Cache tidak memiliki TTL. Setelah migrasi, startup parameters memublikasikan kedua katalog PostgreSQL sebelum membuka HTTP; kegagalan lock/snapshot/publikasi menggagalkan startup. Wajibkan startup tersebut sukses sebelum rollout konsumen. Tidak diperlukan JWT/operator atau refresh manual untuk publikasi awal. CRUD service pengelola mencoba publikasi setelah commit DB; `cacheSynced:false` tetap berarti DB tersimpan dan memerlukan pemulihan publikasi, bukan pengulangan mutasi. Refresh manual tersedia saat berjalan. Tidak ada retry worker/outbox. Namespace sesi dan state recovery tidak dibersihkan saat refresh katalog.

Migrasi 0008 mengganti `parameter_key`/`parameter_value` menjadi `code`/`value`, serta menambahkan `module` wajib (data lama `global`, tanpa default insert baru). Module hanya metadata: lookup Redis tetap code→value, bukan module:code, dan konstanta lookup konsumen tidak berubah. SQL/DTO API parameters lama tidak kompatibel dengan module/code/value; core tetap membaca hash yang sama.

Migrasi 0009 memindahkan katalog ke `parameter.m_system_parameter`/`parameter.m_error_code`, identitas/RBAC ke `portal`, serta template/log pengiriman ke `notification`. HTTP, key Redis, dan metode `ParameterCache` tidak berubah. Siapkan auth/users/notification/parameters dengan histori identik 0001–0009 dan query schema eksplisit, lalu stop/drain seluruh binary lama sebelum migrasi bersama. Objek yang tidak dipindahkan, termasuk enum, `public.refresh_tokens`, sequence publikasi dan histori SQLx, tetap public; role runtime terpisah memerlukan USAGE schema. Core tidak menyimpan/menerapkan migrasi sendiri. Detail urutan, hak database, dan pemulihan: [operations parameters](../../../tmj-rust-parameters/docs/operations.md#coordinated-consumer-cutover).

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
