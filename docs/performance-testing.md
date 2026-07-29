# Backend Performance Testing

This project includes a dependency-free Node benchmark runner:

```powershell
node scripts\perf-test.mjs
```

Default behavior:

- Reads all API operations from `docs/openapi.json`.
- Tests only non-mutating `GET` endpoints.
- Skips `/school/...` routes unless `--include-school-routes` is set.
- Skips JWT or school-token protected routes unless the needed token is provided.
- Skips streams and websockets unless `--include-streams` is set.
- Writes Markdown and JSON reports into `reports/perf/`.

Useful examples:

```powershell
# Public GET endpoints, default load profile
node scripts\perf-test.mjs

# Direct answer: how many requests can the backend handle in one second?
node scripts\perf-test.mjs --capacity

# Test one real API endpoint for one-second capacity
node scripts\perf-test.mjs --capacity --target /database/status --concurrency 1,10,25,50

# Create/login a disposable user and test an authenticated endpoint
node scripts\perf-test.mjs --capacity --target /me --create-test-user

# Test login throughput for a known user
node scripts\perf-test.mjs --capacity --method POST --target /login --body-json '{ "email": "perf@example.com", "password": "PerfTest12345!" }'

# Short smoke/perf pass
node scripts\perf-test.mjs --duration 3000 --concurrency 1,10 --samples 2 --warmup 5

# Generate endpoint coverage without calling the server
node scripts\perf-test.mjs --inventory-only

# Include JWT-protected GET endpoints
$env:AUTH_TOKEN="your-access-token"
node scripts\perf-test.mjs --duration 10000 --concurrency 1,10,25,50

# Include school routes
$env:SCHOOL_TOKEN="your-school-token"
node scripts\perf-test.mjs --include-school-routes --duration 10000 --concurrency 1,10,25,50

# Point at another environment
node scripts\perf-test.mjs --base-url https://api.example.com --duration 15000 --concurrency 10,50,100
```

The headline capacity estimate is the highest concurrency level that keeps:

- `errorRate` near `0%`
- `p95Ms` within your acceptable latency target
- status counts mostly `2xx` or expected application-level `4xx`

Avoid using `--include-mutating` against production data. It enables `POST`, `PUT`, `PATCH`, and `DELETE` operations and requires realistic request bodies before results are meaningful.
