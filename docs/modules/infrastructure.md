# Modul Infrastructure (`src/config/`)

## 1. PostgreSQL (`src/config/database.rs`)
- `init_db(database_url: &str, migrator: &Migrator) -> Result<PgPool, Box<dyn std::error::Error>>`
- Membuka koneksi `PgPoolOptions` dengan `max_connections(10)` lalu menjalankan migrasi SQLx (`migrator.run(&db)`) yang dikirim oleh masing-masing binary service (`&sqlx::migrate!("./migrations")`).

Histori migrasi berada di repo auth/users/notification/parameters dan harus identik hingga 0009; core hanya menjalankan `Migrator` yang diberikan. Migrasi 0008 mengubah struktur parameter menjadi module/code/value, sedangkan 0009 memindahkan tabel domain ke schema portal/parameter/notification. Build semua binary dengan query baru dan hentikan seluruh binary lama sebelum schema move. Setelah itu startup parameters harus sukses memublikasikan katalog sebelum rollout konsumen; `init_db` sendiri tidak melakukan publikasi Redis atau mengatur search_path. Lihat [cutover dan hak schema](../../../tmj-rust-parameters/docs/operations.md#schema-ownership-and-privileges-0009).

## 2. Redis (`src/config/redis.rs`)
- `init_redis(redis_url: &str) -> Result<ConnectionManager, redis::RedisError>`
- Membuka client Redis dan menginisialisasi `redis::aio::ConnectionManager` untuk digunakan oleh `SessionRepository`.

## 3. `JwtAccessSecret` (`src/config/mod.rs`)
- Newtype `JwtAccessSecret(pub String)` agar extractor `AuthenticatedUser` di `tmj-rust-core` dapat mengekstrak secret JWT dari `AppState` service mana pun melalui `FromRef` tanpa mengikat `tmj-rust-core` pada struct `Config` spesifik milik service tertentu.
