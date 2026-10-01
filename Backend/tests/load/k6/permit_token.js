// k6 load test for permit-token verification (integration-gateway).
//
// Target: 5000 RPS, p99 < 50ms.

import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 500 },
    { duration: '1m', target: 5000 },
    { duration: '5m', target: 5000 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(99)<50'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const url = 'http://localhost:8081/v1/integration/verify-permit';
  const res = http.post(url, '{}', {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, {
    'is 200': (r) => r.status === 200,
  });
}
