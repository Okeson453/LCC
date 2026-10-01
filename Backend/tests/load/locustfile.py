"""Locust load test for the API gateway."""

from locust import HttpUser, task, between


class LCCUser(HttpUser):
    wait_time = between(1, 5)

    @task(1)
    def get_briefing(self):
        self.client.get("/v1/briefings/latest")

    @task(3)
    def list_content(self):
        self.client.get("/v1/content/items")

    @task(1)
    def healthz(self):
        self.client.get("/healthz")
