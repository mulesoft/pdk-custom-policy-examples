# Benchmarking custom policies with `pdk-unit`

[`pdk-unit`](https://docs.mulesoft.com/pdk/latest/policies-pdk-unit) runs your
policy in an in-process emulated Flex host. Because a request completes in
microseconds with no Docker, no network, and no real gateway, the harness also
works as a
[Criterion](https://bheisler.github.io/criterion.rs/book/) benchmark: you drive
the policy with `UnitTest::request(...)` inside a `b.iter(...)` loop and
Criterion measures how long each request takes.

This lets you catch performance regressions and compare the cost of different
code paths without deploying anything. This example policy, **`pii-redaction`**,
is built specifically to show the technique: it redacts sensitive data from
requests and takes a **different code path depending on what the request is**,
so each path has its own performance profile worth measuring.

> **On Criterion.** Criterion is used here only as one example of a benchmarking
> framework. It is a third-party crate, not owned or endorsed by MuleSoft, and
> any other benchmarking harness should work just as well — the `pdk-unit` integration
> is the same regardless of which you pick.

> **What the numbers mean.** Each measured iteration includes the `pdk-unit`
> request/host emulation overhead *in addition to* your policy's work. Treat the
> results as **relative** — path A vs path B, or this commit vs the last one —
> not as absolute production latencies. The emulator is not the real Flex data
> plane.

---

## The example: one policy, five flows

`pii-redaction` inspects each request and, based on **the request alone** (its
headers and `content-type` — never a special "benchmark mode" header), runs one
of five flows. That is deliberate: the benchmark should exercise the same
decision path production traffic takes.

| Flow            | Selected when                                   | Work done                                  |
| --------------- | ----------------------------------------------- | ------------------------------------------ |
| `passthrough`   | binary/unknown content type (e.g. `image/png`)  | body left untouched                        |
| `header_redact` | sensitive headers present, no redactable body   | mask configured headers                    |
| `regex_scan`    | `content-type: text/plain`                      | single-pass regex sweep over the body      |
| `json_mask`     | `content-type: application/json`, shallow object| parse JSON + walk top-level fields         |
| `json_deep`     | `content-type: application/json`, nested object | recursive walk over the whole JSON tree    |

Because the flows do measurably different amounts of work, the benchmark shows a
clear cost spread — which is exactly the kind of signal you want when deciding
whether a code path is worth optimizing.

---

## Wiring it up

Four pieces connect a policy crate to a Criterion benchmark. All of them are
already in place in this example — use it as a template.

### 1. Expose the crate as a library

A policy compiles to a `cdylib` (the WASM artifact). A benchmark is a *separate*
crate that needs to `import` your policy, so add `rlib` to the crate type in
`Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Anything the benchmark calls must be public. Here that is the policy's
`configure` entrypoint:

```rust
#[entrypoint]
pub async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> { ... }
```

### 2. Declare the benchmark and add Criterion

```toml
[dev-dependencies]
pdk-unit = { version = "1.9.0" }          # keep in lockstep with your `pdk` version
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "redaction"        # matches benches/redaction.rs
harness = false           # required: Criterion provides its own main()
```

> Keep the `pdk-unit` version equal to the `pdk` version in `[dependencies]`. A
> mismatched pair (e.g. `pdk` 1.9.0 with `pdk-unit` 1.8.0) usually fails to link.

### 3. Define the config and sample requests

The benchmark needs a policy config and one request body per flow. Keep them in
a small `fixtures` module inside the benchmark. Craft each body so it routes to
exactly one flow, so the group measures that flow in isolation.

```rust
// benches/redaction.rs
mod fixtures {
    use serde_json::json;

    pub fn config() -> String { /* JSON matching definition/gcl.yaml */ }
    pub fn json_deep_body() -> String { /* nested payload driving the json_deep flow */ }
}
```

### 4. Write the benchmark

Build a tester once per benchmark function, then issue one request per iteration.
Each `bench_function` isolates a single flow by sending the request that routes
to it:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pdk_unit::{UnitHttpRequest, UnitTest, UnitTestBuilder};

fn tester() -> UnitTest {
    UnitTestBuilder::default()
        .with_config(fixtures::config())
        .with_entrypoint(pii_redaction::configure)
}

fn bench_redaction_flows(c: &mut Criterion) {
    let mut group = c.benchmark_group("pii_redaction");

    group.bench_function("regex_scan", |b| {
        let mut t = tester();
        let body = fixtures::text_body();
        b.iter(|| {
            black_box(t.request(
                UnitHttpRequest::post()
                    .with_header("content-type", "text/plain")
                    .with_body(body.clone()),
            ));
        });
    });

    // ...one bench_function per flow...
    group.finish();
}

criterion_group!(benches, bench_redaction_flows);
criterion_main!(benches);
```

Key points:

- **[`black_box(...)`](https://docs.rs/criterion/latest/criterion/fn.black_box.html)**
  stops the optimizer from deleting work whose result is unused. Wrap the
  `request(...)` call in it.
- **Build the tester outside `b.iter`, do per-iteration work inside.** Everything
  in the closure is measured; everything before it is setup. Cloning the request
  body inside the loop is intentional here so each iteration starts from the same
  input.
- **One `bench_function` per flow**, grouped under one
  `benchmark_group`, so the report lines up the flows side by side.

---

## Running it

```bash
cargo bench --bench redaction                    # full run (accurate, slower)
cargo bench --bench redaction -- regex_scan      # one flow by name
# quick smoke while iterating on the bench itself:
cargo bench --bench redaction -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The flags after `--` are Criterion's own; see the
[command-line options](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html)
for the full set. The emulator emits `Trace:` lines on stderr; filter them out
with `2>/dev/null` if you only want the timings. Criterion also writes an HTML
report to `target/criterion/report/index.html` and, on subsequent runs, prints
the change versus the previous run (`change: [-2.1% +0.4%]`) — that delta is the
point when you use this to guard against regressions.

Sample output from this example (Apple Silicon, relative µs — your absolute
numbers will differ):

```
pii_redaction/passthrough    time:   [~18 µs]
pii_redaction/header_redact  time:   [~15 µs]
pii_redaction/regex_scan     time:   [~28 µs]
pii_redaction/json_mask      time:   [~21 µs]
pii_redaction/json_deep      time:   [~24 µs]
```

Reading it: `header_redact` is the floor — a bodyless request that only rewrites
a few headers. `passthrough` costs a little more because it still streams a
request body through the emulator even though the policy ignores it. The body
flows climb from there, `regex_scan` being the most expensive on this input
because it sweeps a larger text payload. The exact ordering depends on your
fixture sizes; the value is seeing the spread and watching it move across
commits.

---

## Common pitfalls

- **`harness = false` missing** → the benchmark won't accept Criterion's CLI
  flags and `criterion_main!` collides with the default test harness.
- **Forgot `rlib`** → `use pii_redaction::...` fails: a bare `cdylib` cannot be
  imported by another crate.
- **Measuring setup instead of work** → building the tester or the request body
  *inside* `b.iter` inflates and blurs the number. Keep constant setup out of the
  loop.
- **Config that fails to parse** → the policy returns 503 for every request and
  you benchmark the error path. Make sure `fixtures::config()` satisfies every
  `required` field in `definition/gcl.yaml`.
- **Timer-driven policies** → `t.request(...)` does not advance simulated time.
  If your policy does periodic work, call `t.sleep(duration)` to move the clock;
  see the `pdk-unit` documentation for details.

---

## See also

- `benches/redaction.rs` — the full benchmark for this policy.
- [PDK Unit Testing](https://docs.mulesoft.com/pdk/latest/policies-pdk-unit) — the in-process test harness this builds on.
- [Criterion.rs user guide](https://bheisler.github.io/criterion.rs/book/) — the benchmarking framework this uses.
- [Criterion.rs API docs](https://docs.rs/criterion/latest/criterion/) — `Criterion`, `black_box`, `BenchmarkGroup`, and CLI flags.
- [PDK Overview](https://docs.mulesoft.com/pdk/latest/policies-pdk-overview)
