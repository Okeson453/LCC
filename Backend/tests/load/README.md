# Load Tests

Run with:

```bash
locust -f tests/load/locustfile.py --host=http://localhost:8080
```

Targets:
- API Gateway: 500 RPS sustained, p99 < 300ms (non-LLM)
- Compliance Governor: 1000 RPS evaluate, p99 < 200ms
- Permit Token verify: 5000 RPS, p99 < 50ms
