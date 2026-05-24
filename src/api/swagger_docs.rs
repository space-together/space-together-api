use actix_web::{get, http::header, web, HttpResponse, Responder};

const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");

#[get("/docs/openapi.json")]
async fn openapi_json() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json; charset=utf-8"))
        .body(OPENAPI_JSON)
}

#[get("/swagger.json")]
async fn swagger_json() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json; charset=utf-8"))
        .body(OPENAPI_JSON)
}

#[get("/api-docs/openapi.json")]
async fn api_docs_openapi_json() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json; charset=utf-8"))
        .body(OPENAPI_JSON)
}

#[get("/docs")]
async fn swagger_ui() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
        .body(
            r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Space Together API Docs</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
  <style>
    :root {
      color-scheme: light;
      --docs-bg: #f7f7f8;
      --docs-surface: #ffffff;
      --docs-text: #1f2937;
      --docs-muted: #6b7280;
      --docs-border: #d1d5db;
      --docs-code-bg: #f3f4f6;
      --docs-accent: #2563eb;
    }
    body.docs-dark {
      color-scheme: dark;
      --docs-bg: #0f172a;
      --docs-surface: #111827;
      --docs-text: #e5e7eb;
      --docs-muted: #9ca3af;
      --docs-border: #374151;
      --docs-code-bg: #1f2937;
      --docs-accent: #60a5fa;
    }
    html, body { margin: 0; min-height: 100%; background: var(--docs-bg); }
    #swagger-ui { min-height: 100vh; }
    .swagger-ui .topbar { display: none; }
    .theme-toggle {
      position: fixed;
      top: 12px;
      right: 16px;
      z-index: 20;
      border: 1px solid var(--docs-border);
      border-radius: 6px;
      background: var(--docs-surface);
      color: var(--docs-text);
      font: 600 13px system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      padding: 8px 12px;
      cursor: pointer;
      box-shadow: 0 10px 24px rgba(15, 23, 42, 0.12);
    }
    body.docs-dark .swagger-ui,
    body.docs-dark .swagger-ui .scheme-container,
    body.docs-dark .swagger-ui section.models,
    body.docs-dark .swagger-ui .model-box,
    body.docs-dark .swagger-ui .opblock,
    body.docs-dark .swagger-ui .opblock-body,
    body.docs-dark .swagger-ui .responses-inner,
    body.docs-dark .swagger-ui .parameters-container,
    body.docs-dark .swagger-ui .execute-wrapper,
    body.docs-dark .swagger-ui .opblock-section-header,
    body.docs-dark .swagger-ui .dialog-ux .modal-ux,
    body.docs-dark .swagger-ui input,
    body.docs-dark .swagger-ui textarea,
    body.docs-dark .swagger-ui select {
      background: var(--docs-bg);
      color: var(--docs-text);
    }
    body.docs-dark .swagger-ui .scheme-container,
    body.docs-dark .swagger-ui .opblock,
    body.docs-dark .swagger-ui section.models,
    body.docs-dark .swagger-ui .model-box,
    body.docs-dark .swagger-ui input,
    body.docs-dark .swagger-ui textarea,
    body.docs-dark .swagger-ui select,
    body.docs-dark .swagger-ui table tbody tr td {
      border-color: var(--docs-border);
    }
    body.docs-dark .swagger-ui,
    body.docs-dark .swagger-ui .info .title,
    body.docs-dark .swagger-ui .info p,
    body.docs-dark .swagger-ui .info li,
    body.docs-dark .swagger-ui .opblock-tag,
    body.docs-dark .swagger-ui .opblock .opblock-summary-description,
    body.docs-dark .swagger-ui .opblock-description-wrapper p,
    body.docs-dark .swagger-ui .parameter__name,
    body.docs-dark .swagger-ui .parameter__type,
    body.docs-dark .swagger-ui .parameters-col_description,
    body.docs-dark .swagger-ui .response-col_status,
    body.docs-dark .swagger-ui .response-col_description,
    body.docs-dark .swagger-ui .model,
    body.docs-dark .swagger-ui .model-title,
    body.docs-dark .swagger-ui .model-toggle,
    body.docs-dark .swagger-ui label,
    body.docs-dark .swagger-ui table thead tr th,
    body.docs-dark .swagger-ui table tbody tr td {
      color: var(--docs-text);
    }
    body.docs-dark .swagger-ui .tab li,
    body.docs-dark .swagger-ui .markdown code,
    body.docs-dark .swagger-ui .prop-type,
    body.docs-dark .swagger-ui .prop-format,
    body.docs-dark .swagger-ui .parameter__deprecated,
    body.docs-dark .swagger-ui .renderedMarkdown p {
      color: var(--docs-muted);
    }
    body.docs-dark .swagger-ui .highlight-code,
    body.docs-dark .swagger-ui .microlight,
    body.docs-dark .swagger-ui .model-example {
      background: var(--docs-code-bg);
    }
    body.docs-dark .swagger-ui a,
    body.docs-dark .swagger-ui .info a {
      color: var(--docs-accent);
    }
  </style>
</head>
<body>
  <button class="theme-toggle" type="button" aria-label="Switch Swagger theme">Dark mode</button>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    const themeButton = document.querySelector(".theme-toggle");
    const savedTheme = localStorage.getItem("spaceTogetherSwaggerTheme") || "dark";
    const setTheme = (theme) => {
      const dark = theme === "dark";
      document.body.classList.toggle("docs-dark", dark);
      themeButton.textContent = dark ? "Light mode" : "Dark mode";
      localStorage.setItem("spaceTogetherSwaggerTheme", theme);
    };
    themeButton.addEventListener("click", () => {
      setTheme(document.body.classList.contains("docs-dark") ? "light" : "dark");
    });
    setTheme(savedTheme);

    window.ui = SwaggerUIBundle({
      url: "/docs/openapi.json",
      dom_id: "#swagger-ui",
      deepLinking: true,
      displayRequestDuration: true,
      docExpansion: "none",
      filter: true,
      persistAuthorization: true,
      tryItOutEnabled: true
    });
  </script>
</body>
</html>"##,
        )
}

#[get("/swagger-ui")]
async fn swagger_ui_alias() -> impl Responder {
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/docs"))
        .finish()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(openapi_json)
        .service(swagger_json)
        .service(api_docs_openapi_json)
        .service(swagger_ui)
        .service(swagger_ui_alias);
}
