#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";

const args = parseArgs(process.argv.slice(2));
const baseUrl = stripTrailingSlash(args.baseUrl || process.env.BASE_URL || "http://127.0.0.1:4646");
const specPath = resolve(args.spec || "docs/openapi.json");
const capacityOnly = boolArg(args.capacity || args.capacityOnly || process.env.PERF_CAPACITY);
const durationMs = Number(args.duration || process.env.PERF_DURATION_MS || (capacityOnly ? 1_000 : 10_000));
const endpointSamples = Number(args.samples || process.env.PERF_ENDPOINT_SAMPLES || 5);
const warmupRequests = Number(args.warmup || process.env.PERF_WARMUP_REQUESTS || 20);
const concurrencyLevels = String(args.concurrency || process.env.PERF_CONCURRENCY || (capacityOnly ? "1,10,25,50,100,200" : "1,10,25,50"))
  .split(",")
  .map((value) => Number(value.trim()))
  .filter((value) => Number.isFinite(value) && value > 0);
const includeMutating = boolArg(args.includeMutating || process.env.PERF_INCLUDE_MUTATING);
const includeAuthMissing = boolArg(args.includeAuthMissing || process.env.PERF_INCLUDE_AUTH_MISSING);
const includeSchoolRoutes = boolArg(args.includeSchoolRoutes || process.env.PERF_INCLUDE_SCHOOL_ROUTES);
const includeStreams = boolArg(args.includeStreams || process.env.PERF_INCLUDE_STREAMS);
const inventoryOnly = boolArg(args.inventoryOnly || process.env.PERF_INVENTORY_ONLY);
const requestTimeoutMs = Number(args.timeout || process.env.PERF_REQUEST_TIMEOUT_MS || 15_000);
const targetPath = args.target || process.env.PERF_TARGET || "/";
const targetMethod = String(args.method || process.env.PERF_METHOD || "GET").toUpperCase();
const targetBodyJson = args.bodyJson || process.env.PERF_BODY_JSON || "";
const createTestUser = boolArg(args.createTestUser || process.env.PERF_CREATE_TEST_USER);
const testUserEmail = args.testUserEmail || process.env.PERF_TEST_USER_EMAIL || `perf-${Date.now()}@example.com`;
const testUserPassword = args.testUserPassword || process.env.PERF_TEST_USER_PASSWORD || "PerfTest12345!";
const testUserName = args.testUserName || process.env.PERF_TEST_USER_NAME || "Perf Test User";
let authToken = args.authToken || process.env.AUTH_TOKEN || "";
const schoolToken = args.schoolToken || process.env.SCHOOL_TOKEN || "";
const schoolId = args.schoolId || process.env.X_SCHOOL_ID || "";
const outputDir = resolve(args.outputDir || "reports/perf");

const id24 = "000000000000000000000000";
if (createTestUser && !authToken) {
  await assertServerReachable();
  authToken = await createOrLoginTestUser();
  console.log(`Authenticated test user: ${testUserEmail}`);
}

const spec = JSON.parse(await readFile(specPath, "utf8"));
const operations = collectOperations(spec);
const eligible = operations.filter((operation) => operation.eligible);
const skipped = operations.filter((operation) => !operation.eligible);

if (!eligible.length) {
  console.error("No eligible endpoints to test. Provide AUTH_TOKEN/SCHOOL_TOKEN or pass --include-auth-missing.");
  process.exit(1);
}

console.log(`Base URL: ${baseUrl}`);
console.log(`OpenAPI operations: ${operations.length}; eligible for benchmark: ${eligible.length}; skipped: ${skipped.length}`);
console.log(`Concurrency levels: ${concurrencyLevels.join(", ")}; duration per level: ${durationMs}ms`);
if (capacityOnly) console.log(`Capacity target: ${targetMethod} ${targetPath}`);

let endpointResults = [];
let loadResults = [];
if (!inventoryOnly) {
  await assertServerReachable();
  const loadItems = capacityOnly ? [targetOperation()] : eligible;
  await warmup(loadItems);
  endpointResults = capacityOnly ? [] : await sampleEachEndpoint(eligible);
  for (const concurrency of concurrencyLevels) {
    loadResults.push(await loadRun(loadItems, concurrency, durationMs));
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  baseUrl,
  config: {
    durationMs,
    endpointSamples,
    warmupRequests,
    concurrencyLevels,
    includeMutating,
    includeAuthMissing,
    includeSchoolRoutes,
    includeStreams,
    inventoryOnly,
    capacityOnly,
    targetMethod,
    targetPath,
    hasTargetBody: Boolean(targetBodyJson),
    createTestUser,
    testUserEmail: createTestUser ? testUserEmail : undefined,
    requestTimeoutMs,
    hasAuthToken: Boolean(authToken),
    hasSchoolToken: Boolean(schoolToken),
    hasSchoolId: Boolean(schoolId),
  },
  totals: {
    operations: operations.length,
    eligible: eligible.length,
    skipped: skipped.length,
  },
  skipped: skipped.map(({ method, path, skipReason }) => ({ method, path, reason: skipReason })),
  endpointResults,
  loadResults,
};

await mkdir(outputDir, { recursive: true });
const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const jsonPath = resolve(outputDir, `perf-report-${stamp}.json`);
const mdPath = resolve(outputDir, `perf-report-${stamp}.md`);
await writeFile(jsonPath, `${JSON.stringify(report, null, 2)}\n`);
await writeFile(mdPath, renderMarkdown(report));

printSummary(report, jsonPath, mdPath);

function parseArgs(values) {
  const out = {};
  for (let index = 0; index < values.length; index += 1) {
    const item = values[index];
    if (!item.startsWith("--")) continue;
    const [rawKey, rawValue] = item.slice(2).split("=", 2);
    const key = rawKey.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    if (rawValue !== undefined) {
      out[key] = rawValue;
    } else if (values[index + 1] && !values[index + 1].startsWith("--")) {
      out[key] = values[index + 1];
      index += 1;
    } else {
      out[key] = "true";
    }
  }
  return out;
}

function boolArg(value) {
  return ["1", "true", "yes", "on"].includes(String(value || "").toLowerCase());
}

function stripTrailingSlash(value) {
  return value.replace(/\/+$/, "");
}

function collectOperations(openapi) {
  const methods = new Set(["get", "post", "put", "patch", "delete"]);
  const result = [];
  for (const [path, pathItem] of Object.entries(openapi.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!methods.has(method)) continue;
      const security = operation.security || [];
      const needsBearer = security.some((entry) => Object.hasOwn(entry, "bearerAuth"));
      const needsSchool = security.some((entry) => Object.hasOwn(entry, "schoolToken")) || path.startsWith("/school/");
      const isMutating = !["get"].includes(method);
      const isStream = path.includes("/stream") || path.includes("/ws/");
      const hasRequiredBody = Boolean(operation.requestBody?.required);
      const missingAuth = needsBearer && !authToken;
      const missingSchool = needsSchool && !schoolToken;
      let skipReason = "";

      if (isStream && !includeStreams) skipReason = "stream/websocket endpoint";
      else if (isMutating && !includeMutating) skipReason = "mutating method disabled";
      else if (hasRequiredBody && !includeMutating) skipReason = "request body required";
      else if (path.startsWith("/school/") && !includeSchoolRoutes) skipReason = "school route disabled";
      else if ((missingAuth || missingSchool) && !includeAuthMissing) {
        skipReason = `missing ${[missingAuth ? "AUTH_TOKEN" : "", missingSchool ? "SCHOOL_TOKEN" : ""].filter(Boolean).join(" and ")}`;
      }

      result.push({
        method: method.toUpperCase(),
        path,
        summary: operation.summary || "",
        urlPath: materializePath(path),
        query: queryString(operation.parameters || []),
        headers: headersFor(needsBearer, needsSchool),
        eligible: !skipReason,
        skipReason,
      });
    }
  }
  return result.sort((a, b) => `${a.method} ${a.path}`.localeCompare(`${b.method} ${b.path}`));
}

function materializePath(path) {
  return path.replace(/\{([^}]+)\}/g, (_, name) => {
    if (name.includes("count") || name.includes("days")) return "1";
    if (name.includes("username")) return "perf-user";
    if (name.includes("code")) return "PERF";
    return id24;
  });
}

function queryString(parameters) {
  const pairs = [];
  for (const param of parameters) {
    if (param.in !== "query") continue;
    if (param.name === "limit") pairs.push(["limit", "1"]);
    if (param.name === "skip") pairs.push(["skip", "0"]);
    if (param.required) pairs.push([param.name, sampleValue(param)]);
  }
  return pairs.length ? `?${new URLSearchParams(pairs).toString()}` : "";
}

function sampleValue(param) {
  const name = String(param.name || "").toLowerCase();
  if (name.includes("id")) return id24;
  if (name.includes("date") || name === "from" || name === "to") return "2026-01-01T00:00:00Z";
  if (name.includes("count") || name.includes("limit") || name.includes("skip") || name.includes("year")) return "1";
  return "perf";
}

function headersFor(needsBearer, needsSchool) {
  const headers = { accept: "application/json" };
  if (needsBearer && authToken) headers.authorization = `Bearer ${authToken}`;
  if (needsSchool && schoolToken) headers["school-token"] = schoolToken;
  if (schoolId) headers["x-school-id"] = schoolId;
  return headers;
}

function targetOperation() {
  let body = undefined;
  if (targetBodyJson) {
    try {
      body = JSON.stringify(JSON.parse(targetBodyJson));
    } catch {
      throw new Error(`Invalid --body-json value: ${targetBodyJson}`);
    }
  }
  const [pathOnly, query = ""] = targetPath.split("?", 2);
  return {
    method: targetMethod,
    path: pathOnly || "/",
    summary: "Capacity target",
    urlPath: pathOnly || "/",
    query: query ? `?${query}` : "",
    headers: body
      ? { ...headersFor(Boolean(authToken), Boolean(schoolToken)), "content-type": "application/json" }
      : headersFor(Boolean(authToken), Boolean(schoolToken)),
    body,
    eligible: true,
    skipReason: "",
  };
}

async function createOrLoginTestUser() {
  const register = await jsonRequest("/register", {
    method: "POST",
    body: {
      name: testUserName,
      email: testUserEmail,
      password: testUserPassword,
    },
  });

  if (register.status >= 200 && register.status < 300 && register.json?.access_token) {
    return register.json.access_token;
  }

  const login = await jsonRequest("/login", {
    method: "POST",
    body: {
      email: testUserEmail,
      password: testUserPassword,
    },
  });

  if (login.status >= 200 && login.status < 300 && login.json?.access_token) {
    return login.json.access_token;
  }

  throw new Error(
    `Could not create/login test user. register=${register.status} ${JSON.stringify(register.json)} login=${login.status} ${JSON.stringify(login.json)}`,
  );
}

async function jsonRequest(path, options) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), requestTimeoutMs);
  try {
    const response = await fetch(`${baseUrl}${path}`, {
      method: options.method,
      headers: {
        accept: "application/json",
        "content-type": "application/json",
      },
      body: JSON.stringify(options.body),
      signal: controller.signal,
    });
    let json = null;
    try {
      json = await response.json();
    } catch {
      json = null;
    }
    return { status: response.status, json };
  } finally {
    clearTimeout(timeout);
  }
}

async function assertServerReachable() {
  const result = await timedFetch({ method: "GET", urlPath: "/", query: "", headers: { accept: "application/json" } });
  if (result.error) {
    throw new Error(`Server is not reachable at ${baseUrl}: ${result.error}`);
  }
}

async function warmup(items) {
  for (let index = 0; index < warmupRequests; index += 1) {
    await timedFetch(items[index % items.length]);
  }
}

async function sampleEachEndpoint(items) {
  const rows = [];
  for (const item of items) {
    const samples = [];
    for (let index = 0; index < endpointSamples; index += 1) {
      samples.push(await timedFetch(item));
    }
    rows.push(summarizeSamples(item, samples));
  }
  return rows.sort((a, b) => b.p95Ms - a.p95Ms);
}

async function loadRun(items, concurrency, ms) {
  const deadline = performance.now() + ms;
  const samples = [];
  let cursor = 0;
  const workers = Array.from({ length: concurrency }, async () => {
    while (performance.now() < deadline) {
      const item = items[cursor++ % items.length];
      samples.push({ ...await timedFetch(item), operation: `${item.method} ${item.path}` });
    }
  });
  await Promise.all(workers);
  return summarizeLoad(concurrency, ms, samples);
}

async function timedFetch(item) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), requestTimeoutMs);
  const started = performance.now();
  try {
    const response = await fetch(`${baseUrl}${item.urlPath}${item.query}`, {
      method: item.method,
      headers: item.headers,
      body: item.body,
      signal: controller.signal,
    });
    await response.arrayBuffer();
    return { ms: performance.now() - started, status: response.status };
  } catch (error) {
    return { ms: performance.now() - started, status: 0, error: error?.name || String(error) };
  } finally {
    clearTimeout(timeout);
  }
}

function summarizeSamples(item, samples) {
  const latencies = samples.map((sample) => sample.ms);
  const statuses = countBy(samples.map((sample) => String(sample.status)));
  return {
    method: item.method,
    path: item.path,
    summary: item.summary,
    samples: samples.length,
    okRate: ratio(samples.filter((sample) => sample.status >= 200 && sample.status < 400).length, samples.length),
    statusCounts: statuses,
    avgMs: round(avg(latencies)),
    p50Ms: round(percentile(latencies, 50)),
    p95Ms: round(percentile(latencies, 95)),
    p99Ms: round(percentile(latencies, 99)),
    maxMs: round(Math.max(...latencies)),
  };
}

function summarizeLoad(concurrency, ms, samples) {
  const latencies = samples.map((sample) => sample.ms);
  const byOperation = new Map();
  for (const sample of samples) {
    if (!byOperation.has(sample.operation)) byOperation.set(sample.operation, []);
    byOperation.get(sample.operation).push(sample);
  }
  const slowestOperations = [...byOperation.entries()]
    .map(([operation, operationSamples]) => ({
      operation,
      requests: operationSamples.length,
      p95Ms: round(percentile(operationSamples.map((sample) => sample.ms), 95)),
      avgMs: round(avg(operationSamples.map((sample) => sample.ms))),
      statusCounts: countBy(operationSamples.map((sample) => String(sample.status))),
    }))
    .sort((a, b) => b.p95Ms - a.p95Ms)
    .slice(0, 15);

  return {
    concurrency,
    durationMs: ms,
    requests: samples.length,
    rps: round(samples.length / (ms / 1000)),
    okRate: ratio(samples.filter((sample) => sample.status >= 200 && sample.status < 400).length, samples.length),
    errorRate: ratio(samples.filter((sample) => sample.status === 0 || sample.status >= 500).length, samples.length),
    statusCounts: countBy(samples.map((sample) => String(sample.status))),
    avgMs: round(avg(latencies)),
    p50Ms: round(percentile(latencies, 50)),
    p95Ms: round(percentile(latencies, 95)),
    p99Ms: round(percentile(latencies, 99)),
    maxMs: round(Math.max(...latencies)),
    slowestOperations,
  };
}

function avg(values) {
  return values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : 0;
}

function percentile(values, p) {
  if (!values.length) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, Math.min(sorted.length - 1, index))];
}

function countBy(values) {
  return values.reduce((acc, value) => {
    acc[value] = (acc[value] || 0) + 1;
    return acc;
  }, {});
}

function ratio(part, total) {
  return total ? round(part / total) : 0;
}

function round(value) {
  return Math.round((value || 0) * 100) / 100;
}

function renderMarkdown(data) {
  const lines = [];
  lines.push("# Backend Performance Report", "");
  lines.push(`Generated: ${data.generatedAt}`);
  lines.push(`Base URL: \`${data.baseUrl}\``);
  if (data.config.capacityOnly) {
    const best = [...data.loadResults].sort((a, b) => b.rps - a.rps)[0];
    lines.push(`Capacity target: \`${data.config.targetMethod} ${data.config.targetPath}\``);
    if (best) lines.push(`Best 1-second capacity: **${best.rps} requests/sec** at concurrency ${best.concurrency}`);
  }
  lines.push(`Operations: ${data.totals.operations}; tested: ${data.totals.eligible}; skipped: ${data.totals.skipped}`, "");
  lines.push("## Throughput", "");
  if (!data.loadResults.length) {
    lines.push("No live throughput run was executed.");
  } else {
    lines.push("| concurrency | requests | rps | ok rate | error rate | p50 ms | p95 ms | p99 ms | max ms |");
    lines.push("| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const row of data.loadResults) {
      lines.push(`| ${row.concurrency} | ${row.requests} | ${row.rps} | ${pct(row.okRate)} | ${pct(row.errorRate)} | ${row.p50Ms} | ${row.p95Ms} | ${row.p99Ms} | ${row.maxMs} |`);
    }
  }
  lines.push("", "## Slowest Endpoints By Sample P95", "");
  if (!data.endpointResults.length) {
    lines.push("No live endpoint samples were executed.");
  } else {
    lines.push("| endpoint | samples | ok rate | statuses | avg ms | p95 ms | max ms |");
    lines.push("| --- | ---: | ---: | --- | ---: | ---: | ---: |");
    for (const row of data.endpointResults.slice(0, 25)) {
      lines.push(`| \`${row.method} ${row.path}\` | ${row.samples} | ${pct(row.okRate)} | \`${JSON.stringify(row.statusCounts)}\` | ${row.avgMs} | ${row.p95Ms} | ${row.maxMs} |`);
    }
  }
  lines.push("", "## Skipped Endpoint Groups", "");
  const groups = countBy(data.skipped.map((row) => row.reason));
  for (const [reason, count] of Object.entries(groups)) {
    lines.push(`- ${reason}: ${count}`);
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

function pct(value) {
  return `${round(value * 100)}%`;
}

function printSummary(data, jsonPath, mdPath) {
  if (data.loadResults.length) {
    if (data.config.capacityOnly) {
      const best = [...data.loadResults].sort((a, b) => b.rps - a.rps)[0];
      console.log(`\nBest 1-second capacity: ${best.rps} req/s at concurrency ${best.concurrency}`);
    }
    console.log("\nThroughput:");
    for (const row of data.loadResults) {
      console.log(`  c=${row.concurrency}: ${row.rps} req/s, p95=${row.p95Ms}ms, errors=${pct(row.errorRate)}, statuses=${JSON.stringify(row.statusCounts)}`);
    }
    if (data.endpointResults.length) {
      console.log("\nSlowest sampled endpoints:");
      for (const row of data.endpointResults.slice(0, 10)) {
        console.log(`  ${row.method} ${row.path}: p95=${row.p95Ms}ms avg=${row.avgMs}ms statuses=${JSON.stringify(row.statusCounts)}`);
      }
    }
  } else {
    console.log("\nInventory-only report generated; no live requests were sent.");
  }
  console.log(`\nReports:\n  ${jsonPath}\n  ${mdPath}`);
}
