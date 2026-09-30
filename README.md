# Tup

Empty starter workspace for the Tup project.

## Rust Development Environment

This workspace includes a Docker-based development environment for Rust services.

- Development container config: `.devcontainer/devcontainer.json`
- Development image definition: `.devcontainer/Dockerfile`
- Build context ignore rules: `.dockerignore`

The container installs Rust through `rustup` and includes `rustfmt`, `clippy`, `rust-analyzer`, and LLDB support for service development.

## Farmer Registration Service

This project now includes a minimal Axum service for registering farmer businesses.

- Health check: `GET /health`
- Registration endpoint: `POST /farmers/register`
- Database: PostgreSQL via `docker-compose.yml`

### Registered Farmer Data

- Business name
- Farmer name
- Email
- Phone
- Address line
- City
- Region
- Postal code
- Country
- Latitude and longitude
- Service radius in kilometers
- Cultivated crops
- Products for sale

### Run Locally

With Docker:

```bash
cp .env.example .env
docker compose up --build
```

The root `.env` file is used by Docker Compose for local secrets such as `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET`.

### Debug In Devcontainer

To debug line by line inside VS Code:

```bash
Dev Containers: Reopen in Container
```

Then use the Run and Debug panel and choose one of:

- `Debug Tup API`
- `Debug Auth Service`
- `Debug Web Service`
- `Debug All Services`

If you want PostgreSQL locally, start it separately with:

```bash
docker compose up -d db
```

Then launch one of the debug profiles above. The debug tasks now start the Rust binaries directly and do not require Docker in this devcontainer.

When debugging from the devcontainer, set `DATABASE_URL` in the root `.env` file to a database address the container can actually reach. The Compose hostname `db` only works for containers inside the Compose network, and `localhost` inside the devcontainer is the devcontainer itself, not your host Docker engine.

If PostgreSQL is running through Docker Compose on the host, use a host-reachable address in `.env`, such as `postgres://postgres:postgres@host.docker.internal:5432/tup` when your Docker setup provides that hostname.

This devcontainer image does not ship with a Docker CLI or a mounted Docker socket, so Docker-based startup still requires a Docker-capable host or a mounted Docker socket.

Without Docker:

```bash
cp .env.example .env
cargo run
```

### Example Request

```bash
curl -X POST http://localhost:3000/farmers/register \
	-H "Content-Type: application/json" \
	-d '{
		"business_name": "Green Valley Farm",
		"farmer_name": "Amina Ndlovu",
		"email": "amina@example.com",
		"phone": "+27-555-0100",
		"address_line1": "Plot 18 River Road",
		"city": "Makhanda",
		"region": "Eastern Cape",
		"postal_code": "6139",
		"country": "South Africa",
		"latitude": -33.3106,
		"longitude": 26.5256,
		"service_radius_km": 25,
		"cultivated_crops": ["maize", "spinach"],
		"products_for_sale": ["fresh spinach", "maize meal"]
	}'
```

## Simple Platform Diagram

```mermaid
flowchart LR
	U[User] --> S[Search local farmers]
	S --> M[Map with nearby farms]
	M --> F[Select a farmer]
	F --> D[Farmer details page]
	D --> C[Crops cultivated]
	D --> P[Products for sale]
```

This shows a simple web platform flow where a user searches for nearby farmers on a map, selects one, and views the crops they cultivate and the products they sell.

## Farmer Registration Flow

```mermaid
flowchart LR
	A[Farmer creates account] --> B[Register business or farm]
	B --> C[Enter farm location]
	C --> D[Add crops cultivated]
	D --> E[Add products for sale]
	E --> F[Submit for review or publish]
	F --> G[Farm appears on map]
```

### Data Registered During Farmer Onboarding

- Farmer name or business name
- Phone number and email address
- Farm address
- Map pin location using latitude and longitude
- Service area or delivery radius
- Crops cultivated
- Products currently available for sale
- Availability by season or harvest period

### Location Data Needed From the Farmer

To place the farm correctly on the map, the farmer should register:

- Full farm address: village, town, district, region, and postal code if available
- Exact map point: latitude and longitude from a dropped map pin or GPS
- Farm name for display on the map
- Landmark or directions: useful when rural addresses are unclear
- Service radius: how far the farmer delivers or serves customers

The minimum reliable location data is usually a map pin with latitude and longitude plus a readable farm address.

