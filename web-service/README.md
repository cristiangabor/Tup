# Tup Web Service

Leptos SSR frontend for Tup.

## Why Separate

Keep this service separate from `auth-service` and the farmer API.

- The web service scales on page views and SSR load.
- The auth service scales on security-sensitive token and login traffic.
- The farmer API scales on map queries and registration traffic.

This is usually a better container split than combining UI with either backend service.

## Recommended Container Split

Keep the Leptos service in its own container.

- Do not merge it into `auth-service`: auth needs tighter security, different scaling, and a smaller public surface.
- Do not merge it into the farmer API: the UI and data API will scale for different workloads.
- Use the web service as the public SSR entry point, and call `auth-service` plus the farmer API over HTTP.

## SEO And Interactivity

- Use SSR for public pages like landing, search, farmer detail, and crop pages.
- Hydrate client-side interactive features such as Leaflet after the initial server render.
- Keep crawlable farmer content in SSR HTML, not only inside the map widget.

## Live Service Integration

The SSR routes fetch real data from the other services.

- Farmer API base URL: `FARMER_API_URL`
- Auth service base URL: `AUTH_SERVICE_URL`

`/farmers` and `/farmers/:slug` are rendered from the farmer API.
`/dashboard` calls `auth-service` and forwards browser cookies to `/auth/me`.

## Leaflet Integration

The map is rendered directly with `leptos-leaflet` components inside the SSR page shell.

- `MapContainer` holds the map viewport.
- `TileLayer` uses OpenStreetMap tiles.
- `Marker` and `Popup` are generated from farmer coordinates.

## Current Routes

- `/`
- `/farmers`
- `/farmers/:slug`
- `/crops/:crop`
- `/dashboard`
- `/health`

The Leaflet runtime initializes in the browser while the surrounding page content stays server-rendered for SEO.

## Run

```bash
cp .env.example .env
cargo run
```

Service port defaults to `3001`.