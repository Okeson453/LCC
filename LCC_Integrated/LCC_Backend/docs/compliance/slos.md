# SLOs

| Service                  | SLO                                | Target      |
|--------------------------|------------------------------------|-------------|
| Compliance Governor      | evaluate latency p99               | < 200ms     |
| API Gateway              | non-LLM HTTP p99                   | < 300ms     |
| Permit Token verify      | verify latency p99                 | < 50ms      |
| OAuth token refresh      | p95                                | < 1s        |
| Briefing generation      | p95                                | < 3s        |
| AI Worker draft          | p95                                | < 3s        |
| AI Worker score          | p95                                | < 3s        |
| API availability         | monthly                            | ≥ 99.5%     |

All SLOs are mirrored as Prometheus alert rules in
`observability/prometheus/alerts.yaml`.
