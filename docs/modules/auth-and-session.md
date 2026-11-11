# Modul Auth, Token, Password, dan Session

## Komponen

| Simbol | Lokasi | Tanggung Jawab |
| --- | --- | --- |
| `UserRole` | [src/entity/role.rs](../../src/entity/role.rs) | Enum `user` / `admin` untuk SQLx (`user_role`), Serde, dan OpenAPI |
| `Claims` | [src/utils/token.rs](../../src/utils/token.rs) | Payload JWT (`sub`, `jti`, `username`, `role`, `token_type`, `exp`, `iat`) |
| `create_access_token` / `create_refresh_token` / `verify_token` | [src/utils/token.rs](../../src/utils/token.rs) | Penerbitan dan verifikasi signature/expiry JWT HS256 |
| `hash_password` / `verify_password` | [src/utils/password.rs](../../src/utils/password.rs) | Hashing password Argon2 dengan salt acak (`OsRng`) dan verifikasi hash |
| `SessionRepository` | [src/repository/session.rs](../../src/repository/session.rs) | Operasi Redis untuk key `session:user:{id}` (`save_session`, `get_active_session_id`, `get_session_tokens`, `delete_session`, `ping`) |
| `AuthenticatedUser` | [src/middleware/auth.rs](../../src/middleware/auth.rs) | Extractor Axum yang memvalidasi header `Bearer <token>`, memastikan `token_type == "access"`, mem-parsing `user_id: Uuid`, dan mencocokkan `claims.jti` dengan `session_id` aktif di Redis |

## Kontrak sesi Redis

Key `session:user:{user_id}` menyimpan hash fields `session_id`, `refresh_token`, `user_id`, `username`, dan `role`, dengan TTL sesuai masa berlaku refresh token. Login/refresh/logout di auth dan extractor `AuthenticatedUser` pada konsumen bergantung pada kontrak ini. Sumber: [SessionRepository](../../src/repository/session.rs) dan [extractor auth](../../src/middleware/auth.rs).

## Kontrak Integrasi `AppState`

Agar handler di suatu service dapat memakai `AuthenticatedUser`, `AppState` service tersebut wajib menyediakan implementasi:
- `FromRef<AppState> for JwtAccessSecret`
- `FromRef<AppState> for SessionRepository`
