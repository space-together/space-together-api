# AGENTS.md

Guidance for AI agents working in the `space-together-api` repository.

## Project Overview

Space Together is a school collaboration and management system for students,
teachers, parents, school staff, and administrators. This repository is the
backend API. It is a Rust service built with Actix Web and MongoDB.

Sibling repositories in the same workspace:

- `space-together-api`: Rust/Actix backend API and OpenAPI documentation.
- `space-together-web`: Next.js web client.
- `space-together-platform`: Next.js platform/admin-style client.
- `space-together-desktop`: Next.js client wrapped by Tauri.

The API starts from `src/main.rs`, loads environment variables, initializes
logging, connects to MongoDB, installs CORS and tenant middleware, then mounts
all routes through `api::init_routes`.

Default API port: `4646`.

## Repository Structure

- `src/api`: Actix route handlers and route registration. New HTTP endpoints
  usually start here.
- `src/domain`: Data models and DTO-like domain structs serialized with Serde.
- `src/services`: Business logic and persistence orchestration.
- `src/repositories`: Lower-level database access helpers.
- `src/middleware`: Request middleware, including JWT and tenant handling.
- `src/guards`: Role and permission checks.
- `src/config`: App state, database setup, and logging.
- `src/utils`, `src/helpers`, `src/mappers`: Shared utility code.
- `docs/openapi.json`: Public API contract served by Swagger routes.
- `tests`: API tests and supporting test assets.

Important route wiring:

- `src/main.rs` calls `api::init_routes`.
- `src/api/mod.rs` imports API modules and calls each module's `init`.
- Many collection APIs use `crate::utils::route_utils::mount_dual_routes`,
  which mounts both singular/plural or legacy-compatible route forms.
- API documentation is served by `src/api/swagger_docs.rs` from
  `docs/openapi.json` at `/docs/openapi.json`, `/swagger.json`, and
  `/api-docs/openapi.json`. Swagger UI is at `/docs`.

## Backend Patterns

- Keep route handlers thin: parse request data, get the database with
  `get_database(&req, &state)`, call a service, then return an HTTP response.
- Put business rules in `src/services`, not directly in route handlers.
- Keep MongoDB details inside services/repositories unless an existing module
  already does otherwise.
- For database reads, writes, updates, deletes, pagination, aggregation, and
  counts, prefer `src/repositories/base_repo.rs` and its `BaseRepository`
  helpers such as `get_all`, `find_one`, `create`, `update_one_and_fetch`,
  `update_many_and_fetch`, `delete_one`, `aggregate_with_paginate`,
  `aggregate_one`, and `count` before writing direct MongoDB calls.
- Use existing domain structs and `make_partial!` for update payloads when
  adding partial-update models.
- Preserve existing response style: success returns JSON data; validation and
  service errors are returned as JSON with the closest existing status code.
- Protected mutations usually wrap handlers with
  `crate::middleware::jwt_middleware::JwtMiddleware` and use role checks from
  `src/guards`.
- Multi-tenant school data should respect `TenantMiddleware`, `School-Token`,
  and helpers such as `get_database` and `get_school_id_from_request`.
- For create/update/delete behavior that clients need live updates for, follow
  existing `EventService::broadcast_created`, `broadcast_updated`, and
  `broadcast_deleted` patterns.

## OpenAPI Requirement

Whenever an AI agent adds, removes, renames, or changes any public API endpoint,
request body, query parameter, response shape, authentication requirement, or
error behavior, it must update:

- `docs/openapi.json`

Keep the OpenAPI operation, tags, schemas, security, parameters, and examples
aligned with the Rust code in the same commit. This file is not optional; it is
what the running API serves to users and other tools.

## Development Commands

- Run locally: `cargo run`
- Check compilation: `cargo check`
- Run tests: `cargo test`
- Format Rust: `cargo fmt`
- Lint Rust when practical: `cargo clippy`

Use the narrowest verification that proves the change. For docs-only changes,
reviewing the rendered Markdown or JSON validity is enough.

## Git Rules

- Every AI-made file change, even a very small one, must be committed before the
  agent gives its final answer.
- Stage only the files changed for the current task. Do not stage unrelated
  user edits.
- If a task touches multiple sibling repositories, make a separate commit in
  each affected repository.
- Before committing, run `git status --short` and check the diff.
- Use short, clear commit messages, for example:
  `docs: add agent guidance`
  `feat: add student attendance endpoint`
  `fix: handle missing school token`
- Never rewrite history, reset, or discard user changes unless the user clearly
  asks for that exact operation.

## Working Style

- Read the relevant existing module before changing it.
- Prefer existing helpers, service patterns, DTOs, and route conventions over
  inventing new structure.
- Keep changes scoped to the user request.
- If frontend clients must call a changed endpoint, update the matching service
  files in the relevant sibling client repositories.
- Keep secrets out of commits. Do not commit `.env` values or credentials.
- Use ASCII in new files unless the surrounding file already needs Unicode.
