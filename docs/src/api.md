# HTTP API

RTFM includes a RESTful HTTP API for integration with other tools.

## Starting the Server

```bash
# Default (localhost:8080)
rtfm serve

# Custom port
rtfm serve --port 3000

# Bind to all interfaces
rtfm serve --bind 0.0.0.0

# Run in background
rtfm serve --detach
```

## Swagger UI

Interactive API documentation is available at:

```
http://localhost:8080/swagger-ui
```

## Endpoints

### Search Commands

```http
GET /api/search?q={query}&lang={lang}&limit={limit}
```

Parameters:
- `q` (required): Search query
- `lang` (optional): Language filter (en, zh, etc.)
- `limit` (optional): Max results (default: 20)

Example:
```bash
curl "http://localhost:8080/api/search?q=docker&limit=5"
```

Response:
```json
{
  "total": 5,
  "results": [
    {
      "name": "docker",
      "description": "Manage Docker containers and images.",
      "category": "common",
      "lang": "en",
      "score": 15.234
    }
  ],
  "took_ms": 2
}
```

### Get Command

```http
GET /api/commands/{name}?lang={lang}
```

Example:
```bash
curl "http://localhost:8080/api/commands/docker?lang=en"
```

Response:
```json
{
  "name": "docker",
  "description": "Manage Docker containers and images.",
  "category": "common",
  "platform": "common",
  "lang": "en",
  "examples": [
    {
      "description": "List all docker containers",
      "code": "docker ps -a"
    }
  ]
}
```

### List Commands

```http
GET /api/commands?lang={lang}&category={category}&offset={offset}&limit={limit}
```

### Get Metadata

```http
GET /api/metadata
```

Response:
```json
{
  "version": "v2.3",
  "command_count": 3245,
  "last_update": "2024-01-15T10:30:00Z",
  "languages": ["en", "zh"]
}
```

### Import Commands

```http
POST /api/import
Content-Type: multipart/form-data

file: <archive.zip>
```

### Health Check

```http
GET /api/health
```

## CORS

CORS is enabled by default, allowing requests from any origin.

## Authentication

Currently no authentication. For production use, place behind a reverse proxy with authentication.

## Rate Limiting

No built-in rate limiting. Use a reverse proxy if needed.

## Example: Integration with fzf

```bash
# Use API with fzf for fuzzy selection
curl -s "http://localhost:8080/api/search?q=$1&limit=50" | \
  jq -r '.results[].name' | \
  fzf --preview "curl -s http://localhost:8080/api/commands/{}"
```
