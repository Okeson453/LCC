// k6 load test for the Compliance Governor.
//
// Run:
//   k6 run tests/load/k6/governor.js
//
// Target: 1000 RPS sustained, p99 < 200ms.

import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '1m', target: 1000 },
    { duration: '5m', target: 1000 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(99)<200'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const url = 'http://localhost:8080/v1/admin/governor/evaluate';
  const payload = JSON.stringify({
    member_id: '00000000-0000-0000-0000-000000000001',
    action_type: 'post_publish',
    target_kind: 'post',
  });
  const params = {
    headers: { 'Content-Type': 'application/json' },
  };
  const res = http.post(url, payload, params);
  check(res, {
    'is 200': (r) => r.status === 200,
  });
}
