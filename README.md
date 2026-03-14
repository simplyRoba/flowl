<p align="center">
  <img src="ui/static/icon-512.png" alt="flowl logo" width="128" />
</p>

# flowl

[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
![GitHub License](https://img.shields.io/github/license/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Fblob%2Fmain%2FLICENSE)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/simplyRoba/flowl/ci.yml?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Factions%2Fworkflows%2Fci.yml%3Fquery%3Dbranch%253Amain)
[![GitHub release](https://img.shields.io/github/v/release/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Freleases)](https://github.com/simplyRoba/flowl/releases)
[![GitHub issues](https://img.shields.io/github/issues/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Fissues)](https://github.com/simplyRoba/flowl/issues)
![GitHub Repo stars](https://img.shields.io/github/stars/simplyRoba/flowl)

> **flowl** — short for **fl**ower **owl** — /flaʊl/ like "fowl" but with an *l*

A small Rust service that exposes plant care data (watering schedules, care needs, etc.) for integration with Home Assistant and other automation platforms.

## Quick start

### Docker run

```bash
docker run -p 4100:4100 -v flowl-data:/data \
  -e FLOWL_MQTT_DISABLED=true \
  ghcr.io/simplyroba/flowl:latest
```

Open `http://localhost:4100`. Data is persisted in the `flowl-data` volume.

### Docker Compose

A `docker-compose.yml` is included in the repository. Uncomment the environment variables to enable all features.

```bash
docker compose up -d
```

## Configuration

### Project variables

| Variable | Default | Description |
| --- | --- | --- |
| `FLOWL_PORT` | `4100` | HTTP server listen port. |
| `FLOWL_DB_PATH` | `/data/flowl.db` | Filesystem path to the SQLite database. |
| `FLOWL_LOG_LEVEL` | `info` | `tracing` level filter for logs. |
| `FLOWL_MQTT_HOST` | `localhost` | MQTT broker hostname. |
| `FLOWL_MQTT_PORT` | `1883` | MQTT broker port. |
| `FLOWL_MQTT_TOPIC_PREFIX` | `flowl` | Topic prefix used for auto-discovery and plant topics. |
| `FLOWL_MQTT_DISABLED` | `false` | Skip MQTT client, state checker, and publishes when set to `true`. |
| `FLOWL_AI_API_KEY` | — | API key for the OpenAI-compatible AI provider. AI features are disabled when unset. |
| `FLOWL_AI_BASE_URL` | `https://api.openai.com/v1` | Base URL for the AI API. |
| `FLOWL_AI_MODEL` | `gpt-4.1-mini` | Model name used for all AI tasks. |
| `FLOWL_AI_RATE_LIMIT` | `10` | Max AI requests per minute (0 to disable). |

### Compatible AI models

The model must support **vision** (image input), **structured output** (`response_format: json_schema`), and **streaming** (SSE).

#### OpenAI (`https://api.openai.com/v1`)

| Model | Vision | Structured output | Streaming | Notes |
| --- | --- | --- | --- | --- |
| `gpt-5-mini` | Yes | Yes | Yes | GPT-5 family, most capable but significantly more expensive |
| `gpt-4.1-mini` | Yes | Yes | Yes | Successor to gpt-4o-mini with better performance |
| `gpt-4.1-nano` | Yes | Yes | Yes | Cheapest OpenAI option, outperforms gpt-4o-mini |
| `gpt-4o-mini` | Yes | Yes | Yes | Previous generation, still capable |

#### Mistral (`https://api.mistral.ai/v1`)

| Model | Vision | Structured output | Streaming | Notes |
| --- | --- | --- | --- | --- |
| `mistral-small-latest` | Yes | Yes | Yes | Best value — 24B, strong vision and structured output |
| `ministral-14b-latest` | Yes | Yes | Yes | 14B edge model, successor to pixtral-12b, Apache 2.0 |

To use Mistral, set `FLOWL_AI_BASE_URL=https://api.mistral.ai/v1` and `FLOWL_AI_API_KEY` to your Mistral API key.

Any other provider exposing an OpenAI-compatible `/v1/chat/completions` endpoint (LM Studio, vLLM, Ollama with OpenAI shim, etc.) should also work as long as the model supports the capabilities above.

## Security

flowl has no built-in authentication. It is designed to run on a trusted home network or behind a reverse proxy that handles auth (e.g., Authelia, Authentik, Caddy with basic auth). Do not expose it directly to the internet.

## Home Assistant

With MQTT enabled, each plant appears as a sensor entity (`sensor.flowl_<name>`) via auto-discovery. The state is `ok`, `due`, or `overdue`. Attributes include `last_watered`, `next_due`, and `watering_interval_days`.

### Example: thirsty plants notification

```yaml
automation:
  - alias: "Thirsty plants notification"
    trigger:
      - platform: time
        at: "08:00:00"
    condition:
      - condition: template
        value_template: >
          {{ states.sensor
            | selectattr('entity_id', 'match', 'sensor.flowl_')
            | selectattr('state', 'in', ['due', 'overdue'])
            | list | count > 0 }}
    action:
      - service: notify.mobile_app_your_phone
        data:
          title: "Plants need water"
          message: >
            {% set thirsty = states.sensor
              | selectattr('entity_id', 'match', 'sensor.flowl_')
              | selectattr('state', 'in', ['due', 'overdue'])
              | list %}
            {{ thirsty | count }} {{ 'plant needs' if thirsty | count == 1 else 'plants need' }} water:
            {{ thirsty | map(attribute='name') | map('replace', 'flowl ', '') | join(', ') }}
```

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
