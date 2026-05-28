//! Lightweight in-memory request metrics.
//!
//! Records per-route request count, status-class counters, and latency
//! distribution into a fixed bucket histogram, then renders the result in the
//! Prometheus text exposition format at `/metrics`. Designed for first-cut
//! profiling without adding any new dependencies — when crates.io is reachable
//! again, swap for `axum-prometheus` or a real `metrics`/Prometheus client.

use axum::body::Body;
use axum::extract::{MatchedPath, State};
use axum::http::{Request, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::AppState;

/// Prometheus default-style buckets, expressed in microseconds for precision.
/// Last entry is the implicit `+Inf` bucket and is always incremented.
const BUCKET_BOUNDS_US: [u64; 14] = [
    1_000,
    2_500,
    5_000,
    10_000,
    25_000,
    50_000,
    100_000,
    250_000,
    500_000,
    1_000_000,
    2_500_000,
    5_000_000,
    10_000_000,
    u64::MAX,
];

#[derive(Default)]
struct RouteMetrics {
    count: AtomicU64,
    sum_micros: AtomicU64,
    buckets: [AtomicU64; BUCKET_BOUNDS_US.len()],
    status_2xx: AtomicU64,
    status_4xx: AtomicU64,
    status_5xx: AtomicU64,
}

impl RouteMetrics {
    fn record(&self, micros: u64, status: u16) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_micros.fetch_add(micros, Ordering::Relaxed);
        for (i, bound) in BUCKET_BOUNDS_US.iter().enumerate() {
            if micros <= *bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
            }
        }
        match status / 100 {
            2 => self.status_2xx.fetch_add(1, Ordering::Relaxed),
            4 => self.status_4xx.fetch_add(1, Ordering::Relaxed),
            5 => self.status_5xx.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }
}

#[derive(Clone, Default)]
pub struct MetricsRegistry {
    inner: Arc<DashMap<String, Arc<RouteMetrics>>>,
}

impl std::fmt::Debug for MetricsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsRegistry")
            .field("routes", &self.inner.len())
            .finish()
    }
}

impl MetricsRegistry {
    fn record(&self, route: &str, method: &str, micros: u64, status: u16) {
        let key = format!("{method} {route}");
        let entry = self
            .inner
            .entry(key)
            .or_insert_with(|| Arc::new(RouteMetrics::default()))
            .clone();
        entry.record(micros, status);
    }

    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("# HELP http_request_duration_microseconds Request latency by route\n");
        out.push_str("# TYPE http_request_duration_microseconds histogram\n");
        for entry in self.inner.iter() {
            let key = entry.key();
            let m = entry.value();
            let count = m.count.load(Ordering::Relaxed);
            let sum = m.sum_micros.load(Ordering::Relaxed);
            for (i, bound) in BUCKET_BOUNDS_US.iter().enumerate() {
                let v = m.buckets[i].load(Ordering::Relaxed);
                let label = if *bound == u64::MAX {
                    "+Inf".to_string()
                } else {
                    bound.to_string()
                };
                out.push_str(&format!(
                    "http_request_duration_microseconds_bucket{{route=\"{key}\",le=\"{label}\"}} {v}\n"
                ));
            }
            out.push_str(&format!(
                "http_request_duration_microseconds_count{{route=\"{key}\"}} {count}\n"
            ));
            out.push_str(&format!(
                "http_request_duration_microseconds_sum{{route=\"{key}\"}} {sum}\n"
            ));
        }

        out.push_str("# HELP http_requests_total Total requests by route and status class\n");
        out.push_str("# TYPE http_requests_total counter\n");
        for entry in self.inner.iter() {
            let key = entry.key();
            let m = entry.value();
            for (class, count) in [
                ("2xx", m.status_2xx.load(Ordering::Relaxed)),
                ("4xx", m.status_4xx.load(Ordering::Relaxed)),
                ("5xx", m.status_5xx.load(Ordering::Relaxed)),
            ] {
                if count > 0 {
                    out.push_str(&format!(
                        "http_requests_total{{route=\"{key}\",status=\"{class}\"}} {count}\n"
                    ));
                }
            }
        }
        out
    }
}

/// Middleware that times each request and records into the registry. Uses the
/// matched route pattern (e.g. `/api/transactions/{id}`) as the label so labels
/// stay bounded as user-generated path segments come and go.
pub async fn record_request(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());

    let response = next.run(req).await;

    let micros = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
    state
        .metrics
        .record(&route, method.as_str(), micros, response.status().as_u16());

    response
}

/// `/metrics` handler — Prometheus text exposition format.
pub async fn render_metrics(State(state): State<AppState>) -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        state.metrics.render(),
    )
}
