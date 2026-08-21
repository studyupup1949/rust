# Docker Deployment

Run Adversaria in Docker containers.

## Quick Start

### Build Image

```bash
cd docker
docker build -t adversaria:latest -f Dockerfile ..
```

### Run Container

```bash
# List suites
docker run --rm adversaria:latest list

# Run tests
docker run --rm \
  -e OPENAI_API_KEY="your-key" \
  -v $(pwd)/reports:/app/reports \
  adversaria:latest run --provider openai
```

## Docker Compose

### Start Services

```bash
# Set API keys
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"

# Run with docker-compose
docker-compose up adversaria
```

### Run Tests

```bash
docker-compose run test
```

### Ollama Integration

```bash
# Start Ollama
docker-compose up -d ollama

# Pull model
docker-compose exec ollama ollama pull llama2

# Run tests
docker-compose run ollama-test
```

## Environment Variables

- `OPENAI_API_KEY`: OpenAI API key
- `ANTHROPIC_API_KEY`: Anthropic API key
- `RUST_LOG`: Logging level (default: adversaria=info)

## Volumes

- `/app/reports`: Test reports
- `/app/suites`: Attack suites
- `/app/adversaria.config.yaml`: Configuration

## Examples

### Custom Configuration

```bash
docker run --rm \
  -v $(pwd)/custom-config.yaml:/app/adversaria.config.yaml \
  -v $(pwd)/reports:/app/reports \
  adversaria:latest run
```

### Specific Suites

```bash
docker run --rm \
  -e OPENAI_API_KEY="your-key" \
  adversaria:latest run --suites prompt_injection,jailbreak
```

### View Reports

```bash
docker run --rm \
  -v $(pwd)/reports:/app/reports \
  adversaria:latest report --list
```

## CI/CD Integration

### GitHub Actions

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker image
        run: docker build -t adversaria:latest -f docker/Dockerfile .
      - name: Run tests
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: |
          docker run --rm \
            -e OPENAI_API_KEY \
            -v $(pwd)/reports:/app/reports \
            adversaria:latest run
```

### GitLab CI

```yaml
test:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t adversaria:latest -f docker/Dockerfile .
    - docker run --rm -e OPENAI_API_KEY adversaria:latest run
```

## Production Deployment

### Kubernetes

Create `adversaria-cronjob.yaml`:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: adversaria-security-test
spec:
  schedule: "0 0 * * 0"  # Weekly
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: adversaria
            image: adversaria:latest
            env:
            - name: OPENAI_API_KEY
              valueFrom:
                secretKeyRef:
                  name: adversaria-secrets
                  key: openai-api-key
            volumeMounts:
            - name: reports
              mountPath: /app/reports
          volumes:
          - name: reports
            persistentVolumeClaim:
              claimName: adversaria-reports
          restartPolicy: OnFailure
```

Apply:
```bash
kubectl apply -f adversaria-cronjob.yaml
```

## Troubleshooting

### Permission Issues

```bash
# Fix permissions
docker run --rm -v $(pwd)/reports:/app/reports adversaria:latest sh -c "chmod 777 /app/reports"
```

### Network Issues

```bash
# Use host network
docker run --rm --network host adversaria:latest run --provider ollama
```

### Debug Mode

```bash
docker run --rm \
  -e RUST_LOG=adversaria=debug \
  adversaria:latest run
```
