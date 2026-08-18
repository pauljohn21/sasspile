# Spec OTel Metrics + Trace 测试基础设施设计

## 1. 架构总览

### 1.1 当前架构（问题）

```
┌──────────────────────────────────────────────────────────────────┐
│  当前 spec 测试架构                                              │
│                                                                  │
│  tests/spec_runner.rs                                            │
│    ├── run_spec_test() → SpecTestResult                         │
│    └── run_hrx_tests()                                           │
│         ├── 多文件 → SKIPPED (passed=true) ← 假通过!            │
│         └── 单文件 → compile() + normalize_css() 比对            │
│                                                                  │
│  tests/spec_plain.rs (5个HRX, assert)                           │
│  tests/spec_css.rs (统计不assert, 假通过!)                       │
│  tests/spec_directives.rs (统计不assert, 假通过!)                │
│  tests/spec_*.rs (12个文件, 全部统计不assert)                    │
│                                                                  │
│  tests/tracing_init.rs (OTel trace, 只用于真实项目)              │
│  tests/otel_test_harness.rs (Bootstrap/Bulma编译追踪)            │
│                                                                  │
│  问题: 无 metrics, 无量化追踪, 多文件全跳过, 假通过              │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 目标架构

```
┌──────────────────────────────────────────────────────────────────────┐
│  目标架构: Metrics + Trace 双管齐下                                  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │  tests/tracing_init.rs                                      │     │
│  │  ├── init_otel(label)     ← 已有, Trace                    │     │
│  │  ├── shutdown_otel()      ← 已有                           │     │
│  │  ├── init_metrics(label)  ← 新增, Metrics                  │     │
│  │  └── shutdown_metrics()   ← 新增                           │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │  tests/spec_otel_runner.rs (新)                              │     │
│  │  ├── run_spec_test_with_metrics()                          │     │
│  │  │   ├── compile(input) → 比较 output/error               │     │
│  │  │   ├── Counter.add(domain, result)                       │     │
│  │  │   ├── Histogram.record(domain, elapsed_ms)             │     │
│  │  │   ├── 失败时 tracing::error!(不panic)                   │     │
│  │  │   └── span 记录调用链 (compile_pipeline内部span自动桥接) │     │
│  │  ├── run_hrx_tests_with_metrics()                         │     │
│  │  │   ├── 单文件 → 正常运行                                 │     │
│  │  │   └── 多文件 → VFS模式运行 (不再SKIPPED)                │     │
│  │  ├── finalize_metrics() → ObservableGauge 注册通过率       │     │
│  │  └── assert_results() → 全跑完后统一assert                 │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │  tests/hrx_vfs.rs (新)                                       │     │
│  │  ├── HrxVfs { files: HashMap<String, String> }            │     │
│  │  ├── parse_hrx_to_vfs(content) → HrxVfs                   │     │
│  │  └── VfsResolver implements ModuleResolver                 │     │
│  │      └── resolve(url) → 从HashMap查找, tokenize+parse     │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │  tests/spec_baseline.rs (新)                                │     │
│  │  ├── #[test] #[ignore] test_baseline_all()                │     │
│  │  │   ├── 遍历全部1306个HRX                                  │     │
│  │  │   ├── init_metrics + init_otel                          │     │
│  │  │   ├── RecordOnly模式跑完                                 │     │
│  │  │   ├── shutdown → flush到jsonl                            │     │
│  │  │   └── 产出 spec_baseline_{timestamp}.json               │     │
│  │  └── 不panic, 只记录                                        │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │  scripts/spec_diff.rs (rust-script, 新)                     │     │
│  │  ├── 读取两个baseline JSON                                  │     │
│  │  ├── 对比 per-domain: 新增通过/新增失败/回归                │     │
│  │  └── 输出 Markdown 报告                                     │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  产出物:                                                             │
│  otel-metrics-spec_{label}.jsonl  ← Metrics (Counter/Gauge/Hist)    │
│  otel-trace-spec_{label}.jsonl    ← Trace (span树)                  │
│  otel-trace-spec_{label}.events.jsonl ← tracing events              │
│  spec_baseline_{timestamp}.json   ← 结构化基线                       │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

## 2. Metrics 设计

### 2.1 仪表盘指标定义

| 指标名 | 类型 | 标签 | 描述 |
|--------|------|------|------|
| `spec_tests_total` | Counter(u64) | `domain`, `result`(pass/fail/skip) | 累计测试用例计数 |
| `spec_pass_rate` | ObservableGauge(f64) | `domain` | 通过率 0.0-1.0，含 OVERALL |
| `spec_test_duration_ms` | Histogram(f64) | `domain` | 单个用例编译耗时分布 |
| `spec_regression_count` | UpDownCounter(i64) | `domain` | 对比上次baseline的净变化 |

### 2.2 Metrics 初始化

```rust
pub fn init_metrics(label: &str) {
    METRICS_INIT.call_once(|| {
        let exporter = opentelemetry_stdout::MetricExporter::default();
        let resource = opentelemetry_sdk::Resource::builder()
            .with_service_name("sasspile-spec")
            .build();
        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(exporter)
            .with_resource(resource)
            .build();

        opentelemetry::global::set_meter_provider(meter_provider.clone());
        METER_PROVIDER.set(meter_provider).ok();
    });
}

pub fn shutdown_metrics() {
    if let Some(provider) = METER_PROVIDER.get() {
        let _ = provider.force_flush();
        let _ = provider.shutdown();
    }
}
```

### 2.3 Metrics 记录流程

```
run_spec_test_with_metrics(name, domain, input, expected, expected_error, meter):
  ┌──────────────────────────────────────────────────────┐
  │ 1. 创建 Counter + Histogram 实例 (meter缓存)          │
  │ 2. start = Instant::now()                             │
  │ 3. span = spec_span!("spec_test", domain, name)       │
  │ 4. enter span                                          │
  │ 5. result = compile(input) → 比对                     │
  │ 6. elapsed = start.elapsed()                          │
  │ 7. counter.add(1, [domain, result])                   │
  │ 8. histogram.record(elapsed_ms, [domain])              │
  │ 9. if fail: tracing::error!(domain, name, mismatch)  │
  │    → error 级别 event 写入 events.jsonl              │
  │    → 不panic, 继续下一个                               │
  │ 10. exit span → OTel span 自动 flush                  │
  └──────────────────────────────────────────────────────┘
```

### 2.4 ObservableGauge 注册

```rust
// 在所有测试跑完后, 注册 ObservableGauge 报告通过率
meter.register_callback(
    &[move |observer| {
        for (domain, stats) in &domain_stats {
            let rate = stats.passed as f64 / stats.total as f64;
            observer.observe_f64(rate, [KeyValue::new("domain", domain)]);
        }
        // OVERALL
        let overall = total_passed as f64 / total as f64;
        observer.observe_f64(overall, [KeyValue::new("domain", "OVERALL")]);
    }],
    "spec_pass_rate",
);
```

## 3. Trace 证据链设计

### 3.1 核心原则：error! 不 panic，trace 先 flush，最后统一 assert

```
    init_otel(label)
    │
    │  for each HRX test case:
    │    span = spec_span!("spec_test", domain, name)
    │    ├── Ok + match → tracing::info!(result="pass")
    │    ├── Ok + mismatch → tracing::error!(result="fail", expected, actual)
    │    │                  ← 不panic! 继续跑
    │    ├── Err(compile_error) → tracing::error!(result="error", error=%e)
    │    │                        ← 不panic! 继续跑
    │    └── span exit → OTel span 自动记录
    │
    │  shutdown_otel() → 强制flush所有pending spans
    │  shutdown_metrics() → 强制flush所有metrics
    │
    ▼
    if total_failed > 0:
        panic!("{} / {} tests failed. See:
            otel-trace-spec_{}.jsonl
            otel-metrics-spec_{}.jsonl", total_failed, total, label)
```

### 3.2 失败用例的 span 调用链示例

```
spec_test (domain=css/plain, test=slash_0, result=fail)
├── compile_pipeline (stage=compile, elapsed_ms=2)
│   ├── tokenize (stage=lexer, elapsed_ms=0)
│   ├── parse (stage=parser, elapsed_ms=1)
│   │   └── parse_declaration (property=color, value=...)
│   ├── evaluate (stage=eval, elapsed_ms=1)
│   │   ├── eval_stmt (stmt=VariableDecl, name=$x)
│   │   ├── eval_expr (expr=Operation, op=Div)
│   │   │   └── ⚠️ eval_div: slash vs div ambiguity
│   │   │       expected: "10px / 20px" (slash-separated)
│   │   │       actual:   "0.5px" (computed division)
│   │   │       → ROOT CAUSE
│   │   └── eval_stmt (stmt=Declaration)
│   └── serialize (stage=serialize, elapsed_ms=0)
└── result=fail, mismatch="expected: a { color: 10px/20px }
    actual: a { color: 0.5px }"
```

### 3.3 Tracing Span 设计

| Span 名称 | Stage | 关键字段 | 描述 |
|-----------|-------|---------|------|
| `spec_test` | spec_test | `domain`, `test_name`, `result` | 顶层测试用例 span |
| `spec_hrx` | spec_test | `hrx_file`, `case_count` | 每个 HRX 文件 span |
| `spec_domain` | spec_test | `domain`, `hrx_count` | 每个域 span |
| `spec_baseline` | spec_test | `total_hrx`, `total_cases` | 全量基线 span |

（compile_pipeline 内部的 span 由 `src/lib.rs` 的 `#[instrument]` 自动桥接，不需手动插桩）

## 4. HRX VFS 设计

### 4.1 数据结构

```rust
/// HRX 内存文件系统 — 解析 HRX 中所有文件到 HashMap
pub struct HrxVfs {
    /// 路径 → 文件内容 (如 "subdir/_partial.scss" → "a { color: red; }")
    files: HashMap<String, String>,
    /// 输入文件路径 (如 "input.scss")
    input_path: String,
}

/// VFS 模块解析器 — 实现 ModuleResolver trait
pub struct VfsResolver {
    vfs: HrxVfs,
    /// 解析后的 AST 缓存
    ast_cache: HashMap<String, Vec<Stmt>>,
}
```

### 4.2 VfsResolver 实现

```rust
impl ModuleResolver for VfsResolver {
    fn resolve(&mut self, url: &str, _base_dir: &Path) -> Result<ResolvedModule, SassError> {
        // 1. 尝试精确匹配: url → vfs.files[url]
        // 2. 尝试加 .scss: url → vfs.files[url + ".scss"]
        // 3. 尝试加 _: url/basename → vfs.files[url/_basename + ".scss"]
        // 4. 尝试 .css (is_css=true): url → vfs.files[url + ".css"]

        let content = self.vfs.files.get(&resolved_path)
            .ok_or_else(|| SassError::eval(
                format!("Module not found: {}", url),
                SourcePos::default(),
            ))?;

        // tokenize + parse (或从 ast_cache 取)
        let ast = self.ast_cache.get(&resolved_path)
            .cloned()
            .unwrap_or_else(|| {
                let tokens = tokenize(content, &resolved_path).unwrap();
                let ast = parse(tokens).unwrap();
                self.ast_cache.insert(resolved_path.clone(), ast.clone());
                ast
            });

        Ok(ResolvedModule {
            ast,
            is_css: resolved_path.ends_with(".css"),
            raw_content: if resolved_path.ends_with(".css") {
                Some(content.clone())
            } else {
                None
            },
            source_path: PathBuf::from(&resolved_path),
        })
    }
}
```

### 4.3 多文件测试解锁流程

```
run_hrx_tests_with_metrics(hrx_path, domain, meter):
  ┌──────────────────────────────────────────────────────┐
  │ 1. content = read_to_string(hrx_path)                │
  │ 2. files = parse_hrx(content)                        │
  │ 3. cases = extract_test_cases(hrx_path)              │
  │ 4. for case in cases:                                 │
  │      if is_single_file_case(case):                    │
  │        → run_spec_test_with_metrics(单文件模式)       │
  │      else:                                             │
  │        → run_multi_file_test_with_metrics(VFS模式)    │
  │           vfs = build_vfs_from_case(case, files)      │
  │           resolver = VfsResolver::new(vfs)            │
  │           input = vfs.get("input.scss")               │
  │           result = compile_with_vfs(input, resolver)  │
  │           → 比较 output                               │
  └──────────────────────────────────────────────────────┘
```

### 4.4 compile_with_vfs 入口

需要 `src/lib.rs` 新增公共 API（不改编译器逻辑，只加入口）：

```rust
pub fn compile_with_resolver(
    source: &str,
    resolver: &mut dyn ModuleResolver,
) -> Result<String, SassError> {
    let tokens = tokenize(source, "<string>")?;
    let ast = parse(tokens)?;
    let css_tree = evaluate(ast, resolver)?;
    Ok(serialize(&css_tree)?)
}
```

## 5. 测试文件改造方案

### 5.1 统一模式

所有 17 个 `spec_*.rs` 改为：

```rust
#[path = "hrx_parser.rs"] mod hrx_parser;
#[path = "hrx_vfs.rs"] mod hrx_vfs;
#[path = "spec_otel_runner.rs"] mod spec_otel_runner;
mod tracing_init;

#[test]
fn test_{domain}_otel() {
    tracing_init::init_otel("spec_{domain}");
    tracing_init::init_metrics("spec_{domain}");

    let dir = spec_root().join("{domain_path}");
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut runner = spec_otel_runner::SpecOtelRunner::new("{domain}", &meter);

    for hrx_path in &hrx_files {
        runner.run_hrx_tests(hrx_path);
    }

    let stats = runner.finalize();
    tracing_init::shutdown_metrics();
    tracing_init::shutdown_otel();

    // 统一 assert
    if stats.failed > 0 {
        panic!(
            "{} / {} tests failed in domain '{}'. \
             See otel-trace-spec_{}.jsonl and otel-metrics-spec_{}.jsonl",
            stats.failed, stats.total, "{domain}",
            "spec_{domain}", "spec_{domain}"
        );
    }
}
```

### 5.2 文件映射

| 现有文件 | 改造方式 |
|---------|---------|
| `spec_plain.rs` (5个HRX, assert) | 改用 SpecOtelRunner |
| `spec_css.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_directives.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_expressions.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_operators.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_parser.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_values.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_variables.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_callable.rs` (统计不assert) | 改用 SpecOtelRunner + assert |
| `spec_core_functions_color.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_list.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_map.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_math.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_meta.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_string.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_selector.rs` | 改用 SpecOtelRunner + assert |
| `spec_core_functions_misc.rs` | 改用 SpecOtelRunner + assert |
| `spec_runner.rs` | 保留，SpecOtelRunner 内部调用 |
| `hrx_parser.rs` | 保留，SpecOtelRunner 内部调用 |
| `hrx_verify.rs` | 保留 |

### 5.3 spec_baseline.rs

```rust
//! 全量 sass-spec 基线测试
//! 运行: cargo test --test spec_baseline -- --nocapture --ignored

#[path = "hrx_parser.rs"] mod hrx_parser;
#[path = "hrx_vfs.rs"] mod hrx_vfs;
#[path = "spec_otel_runner.rs"] mod spec_otel_runner;
mod tracing_init;

#[test]
#[ignore]
fn test_baseline_all() {
    tracing_init::init_otel("spec_baseline");
    tracing_init::init_metrics("spec_baseline");

    let spec_dir = spec_root();
    let all_hrx = find_all_hrx(&spec_dir); // 排除 libsass, non_conformant

    let mut runner = spec_otel_runner::SpecOtelRunner::new("baseline", &meter);

    let domains = [
        ("css/plain", "css_plain"),
        ("css/selector", "css_selector"),
        ("css/media", "css_media"),
        ("css/supports", "css_supports"),
        ("css/custom_properties", "css_custom_properties"),
        ("css/functions", "css_functions"),
        ("css/moz_document", "css_moz_document"),
        ("css/unicode_range", "css_unicode_range"),
        ("css/unknown_directive", "css_unknown_directive"),
        ("directives", "directives"),
        ("expressions", "expressions"),
        ("operators", "operators"),
        ("parser", "parser"),
        ("values", "values"),
        ("variables", "variables"),
        ("callable", "callable"),
        ("core_functions", "core_functions"),
    ];

    for (path, label) in &domains {
        let dir = spec_dir.join(path);
        if dir.exists() {
            let hrx_files = hrx_parser::find_hrx_files(&dir);
            for hrx_path in &hrx_files {
                runner.run_hrx_tests_with_domain(hrx_path, label);
            }
        }
    }

    let stats = runner.finalize();
    runner.write_baseline_json(&stats);

    tracing_init::shutdown_metrics();
    tracing_init::shutdown_otel();

    // 不panic — baseline 模式只记录
    tracing::info!(
        stage = "spec_baseline",
        total = stats.total,
        passed = stats.passed,
        failed = stats.failed,
        skipped = stats.skipped,
        pass_rate = stats.pass_rate(),
        "Baseline complete"
    );
}
```

## 6. Baseline JSON 格式

```json
{
  "timestamp": "2026-08-18T16:00:00Z",
  "version": "0.9.3",
  "total": 1306,
  "passed": 452,
  "failed": 654,
  "skipped": 200,
  "pass_rate": 0.346,
  "domains": {
    "css_plain": { "total": 45, "passed": 38, "failed": 5, "skipped": 2 },
    "css_selector": { "total": 17, "passed": 8, "failed": 7, "skipped": 2 },
    "directives": { "total": 157, "passed": 30, "failed": 100, "skipped": 27 },
    "core_functions": { "total": 874, "passed": 120, "failed": 654, "skipped": 100 }
  },
  "failed_tests": [
    {
      "domain": "css_plain",
      "name": "slash_0",
      "trace_span_id": "abc123",
      "mismatch": "expected: ... actual: ..."
    }
  ]
}
```

## 7. Tracing Span 设计（跨函数管道）

| Span 名称 | Stage | 关键字段 | 触发位置 |
|-----------|-------|---------|---------|
| `spec_baseline` | spec_test | `total_hrx`, `total_cases` | spec_baseline.rs 顶层 |
| `spec_domain` | spec_test | `domain`, `hrx_count` | 每个域开始 |
| `spec_hrx` | spec_test | `hrx_file`, `case_count` | 每个 HRX 文件 |
| `spec_test` | spec_test | `domain`, `test_name`, `result` | 每个测试用例 |
| `compile_pipeline` | compile | (已有) | src/lib.rs #[instrument] |
| `tokenize` | lexer | (已有) | src/lexer.rs |
| `parse` | parser | (已有) | src/parser/mod.rs |
| `evaluate` | eval | (已有) | src/eval/mod.rs |
| `serialize` | serialize | (已有) | src/serialize.rs |

## 8. Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| Metrics SDK 需要异步 runtime | `SdkMeterProvider::with_periodic_exporter` 内部用 tokio，但 `force_flush` 同步等待 |
| `ObservableGauge` 需要回调注册 | 在 `finalize()` 中一次性注册，所有测试跑完后回调 |
| VFS 模块解析可能找不到文件 | 实现 Sass 标准查找逻辑：精确 → .scss → _partial.scss → .css |
| 1306 个 HRX 全跑可能很慢 | baseline 模式 `#[ignore]` 默认不跑；Histogram 找到慢域；可加 `--filter` |
| 失败用例的 trace 文件可能巨大 | `RUST_LOG=info` 默认级别过滤；trace 级别手动开启 |
| `opentelemetry-stdout` metrics feature | 确认 Cargo.toml 加 `"metrics"` feature |
| compile_with_resolver 需要新增公共API | 只在 lib.rs 加一层 wrapper，不改编译器逻辑 |

## 9. 实现顺序依赖

```
Task 1 (Cargo.toml) ─────┐
                          ▼
Task 2 (tracing_init.rs) ──┐
                           ▼
Task 3 (hrx_vfs.rs) ────────┐
                            ▼
Task 4 (spec_otel_runner.rs) ─┐
                               ▼
Task 5 (spec_baseline.rs) ────┐
                               ▼
Task 6 (改造17个spec_*.rs) ────┐
                               ▼
Task 7 (spec_diff.rs) ────────┐
                               ▼
Task 8 (首次基线运行) ─────────┘
```
