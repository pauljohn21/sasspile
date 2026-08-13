//! 企业级基准测试：Bootstrap + Element Plus + Dart Sass 对比
//!
//! 运行：cargo bench --bench enterprise_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn bs_scss(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bs")
        .join("scss")
        .join(file)
}

fn ep_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ep")
        .join("packages")
        .join("theme-chalk")
        .join("src")
}

// —— sasspile 编译 Bootstrap ——

fn bench_sasspile_bootstrap_reboot(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap-reboot.scss")).unwrap_or_default();

    c.bench_function("sasspile_bootstrap_reboot_48kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&input), sasspile::OutputStyle::Expanded));
        })
    });
}

fn bench_sasspile_bootstrap_main(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap.scss")).unwrap_or_default();

    c.bench_function("sasspile_bootstrap_main_158kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&input), sasspile::OutputStyle::Expanded));
        })
    });
}

// —— sasspile 编译 Element Plus ——

fn bench_sasspile_element_plus(c: &mut Criterion) {
    let dir = ep_src();
    let mut combined = String::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
            .collect();
        files.sort_by_key(|e| e.path());

        for entry in &files {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                combined.push_str(&content);
                combined.push('\n');
            }
        }
    }

    if combined.is_empty() {
        // 如果找不到文件，用一个简单的输入
        combined = ".a { color: red; }".repeat(100);
    }

    c.bench_function("sasspile_element_plus_200kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&combined), sasspile::OutputStyle::Expanded));
        })
    });
}

// —— Dart Sass 对比（如果可用）——

fn bench_dart_sass_bootstrap(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap.scss")).unwrap_or_default();

    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("bench_bs_input.scss");
    let output_path = temp_dir.join("bench_bs_output.css");

    std::fs::write(&input_path, &input).ok();

    let sass_bin = "/opt/homebrew/opt/dart-sass/bin/sass";
    let sass_available = Command::new(sass_bin)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !sass_available {
        return;
    }

    c.bench_function("dart_sass_bootstrap_main_158kb", |b| {
        b.iter(|| {
            let _ = Command::new(sass_bin)
                .arg("--style=expanded")
                .arg(&input_path)
                .arg(&output_path)
                .output();
        })
    });
}

criterion_group! {
    name = enterprise_bench;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3)).sample_size(10);
    targets = bench_sasspile_bootstrap_reboot, bench_sasspile_bootstrap_main, bench_sasspile_element_plus, bench_dart_sass_bootstrap
}
criterion_main!(enterprise_bench);
