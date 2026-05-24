# Database Seeding

Use the seed command when you create or switch to a new MongoDB database and
want the base records inserted automatically.

```bash
cargo run --bin seed
```

The command reads the same `.env` values as the API:

- `MONGO_URI`
- `MAIN_DB_NAME`

To seed a different database without changing `MAIN_DB_NAME`, set
`SEED_DB_NAME`:

```bash
SEED_DB_NAME=space_together_dev cargo run --bin seed
```

On PowerShell:

```powershell
$env:SEED_DB_NAME="space_together_dev"; cargo run --bin seed
```

The seed is safe to run many times. It uses stable fields like `username`,
`code`, and `name` to update existing records instead of inserting duplicates.

Seeded data:

- `sectors`
- `main_classes`
- `template_subjects`
- `roles`
