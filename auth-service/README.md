# Tup Auth Service

Standalone authentication and authorization service for Tup. Designed to run independently from other APIs so it can scale as separate pods.

## Security Model

- Cookie-based JWT auth: short-lived access token + rotating refresh token
- CSRF protection: double-submit token (`csrf_token` cookie + `X-CSRF-Token` header)
- Secure cookie controls: `HttpOnly`, `Secure`, `SameSite`, optional cookie domain
- Refresh token rotation: refresh JWT `jti` persisted in PostgreSQL and revoked on use
- Google SSO: OAuth2 Authorization Code flow endpoints

## Endpoints

- `GET /health`
- `GET /auth/csrf`
- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/refresh`
- `POST /auth/logout`
- `GET /auth/me`
- `POST /auth/authorize`
- `GET /auth/google/login`
- `GET /auth/google/callback`

## API Documentation

- Swagger UI: `GET /docs`
- OpenAPI JSON: `GET /api-docs/openapi.json`

The OpenAPI schema documents request and response bodies for the auth and authorization routes, including common error responses.

## Database Migrations

Schema changes are managed through SQL migrations in `migrations/`.

- Initial migration: `migrations/0001_init_auth.sql`
- Migrations are applied automatically on service startup via `sqlx::migrate!`.

## Run

```bash
cp .env.example .env
cargo run
```

By default it listens on port `3002`.

## Scaling Implications

- Service is stateless by design; auth state is in Postgres (`refresh_tokens`, `users`, `permissions`).
- Multiple pods can serve traffic as long as all pods share the same database and secrets.
- Use the same `JWT_ACCESS_SECRET` and `JWT_REFRESH_SECRET` across all replicas.
- Add Redis only if you later need high-volume token/session introspection beyond DB-backed rotation.