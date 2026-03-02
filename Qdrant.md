```yaml
services:
  qdrant:
    image: qdrant/qdrant:latest
    container_name: qdrant
    ports:
      - "6333:6333"   # REST API
      - "6334:6334"   # gRPC API
    volumes:
      - qdrant_storage:/qdrant/storage
    environment:
      - QDRANT__SERVICE__HTTP_PORT=6333
      - QDRANT__SERVICE__GRPC_PORT=6334
      # Uncomment to enable API key authentication:
      # - QDRANT__SERVICE__API_KEY=your_secret_api_key
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:6333/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

volumes:
  qdrant_storage:
    driver: local
```

**Save as `docker-compose.yml` and run:**

```bash
docker compose up -d
```

**Key details:**

| Endpoint | URL |
|----------|-----|
| REST API | `http://localhost:6333` |
| Web UI (Dashboard) | `http://localhost:6333/dashboard` |
| gRPC | `localhost:6334` |

**Common next steps:**

- **Enable auth** — uncomment `QDRANT__SERVICE__API_KEY` and set a secret
- **Limit memory** — add `mem_limit: 2g` under the service
- **Custom config** — mount a config file:
  ```yaml
  volumes:
    - ./config.yaml:/qdrant/config/production.yaml
  ```