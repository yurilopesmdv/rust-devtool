from time import perf_counter

from prometheus_client import CollectorRegistry, Counter, Histogram, generate_latest
from prometheus_client.exposition import CONTENT_TYPE_LATEST
from starlette.responses import Response


def configure_metrics(app):
    registry = CollectorRegistry()
    requests = Counter(
        "http_requests_total",
        "Requisições HTTP",
        ["method", "route", "status"],
        registry=registry,
    )
    duration = Histogram(
        "http_request_duration_seconds",
        "Duração HTTP",
        ["method", "route"],
        registry=registry,
    )

    @app.middleware("http")
    async def track(request, call_next):
        start = perf_counter()
        status = 500
        try:
            response = await call_next(request)
            status = response.status_code
            return response
        finally:
            route = getattr(request.scope.get("route"), "path", "unmatched")
            method = (
                request.method
                if request.method in {"GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"}
                else "OTHER"
            )
            if route != "/metrics":
                requests.labels(method, route, status).inc()
                duration.labels(method, route).observe(perf_counter() - start)

    @app.get("/metrics", include_in_schema=False)
    async def metrics():
        return Response(generate_latest(registry), headers={"Content-Type": CONTENT_TYPE_LATEST})
