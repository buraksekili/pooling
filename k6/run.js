import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend } from "k6/metrics";

const pooledReqDuration = new Trend("pooled_req_duration");
const spawningReqDuration = new Trend("spawning_req_duration");
const pooledFailRate = new Rate("pooled_fail_rate");
const spawningFailRate = new Rate("spawning_fail_rate");
const pooledReqRate = new Rate("pooled_req_rate");
const spawningReqRate = new Rate("spawning_req_rate");

const myStages = [
  { duration: "5s", target: 200 },
  { duration: "5s", target: 120},
];

export const options = {
  scenarios: {
    pooled_server: {
      executor: "ramping-vus",
      startVUs: 50,
      stages: myStages,
      exec: "testPooledServer",
    },
    spawning_server: {
      executor: "ramping-vus",
      startVUs: 50,
      stages: myStages,
      exec: "testSpawningServer",
    },
  },
};

export function testPooledServer() {
  const res = http.get("http://127.0.0.1:7878");
  pooledReqDuration.add(res.timings.duration);
  pooledReqRate.add(1);
  check(res, { "pooled status was 200": (r) => r.status == 200 });
  pooledFailRate.add(res.status !== 200);
  sleep(0.1);
}

export function testSpawningServer() {
  const res = http.get("http://127.0.0.1:7879");
  spawningReqDuration.add(res.timings.duration);
  spawningReqRate.add(1);
  check(res, { "spawning status was 200": (r) => r.status == 200 });
  spawningFailRate.add(res.status !== 200);
  sleep(0.1);
}
