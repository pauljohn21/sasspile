//! Bootstrap 企业级基准测试
//!
//! 用 Bootstrap 5.3.8 的 SCSS 文件测试 sasspile 编译性能
//!
//! 运行：cargo bench --bench bootstrap_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use std::time::Duration;

fn bs_scss(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bs")
        .join("scss")
        .join(file)
}

fn bench_bootstrap_reboot(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap-reboot.scss")).unwrap_or_default();

    c.bench_function("sasspile_bootstrap_reboot_48kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&input), sasspile::OutputStyle::Expanded));
        })
    });
}

fn bench_bootstrap_grid(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap-grid.scss")).unwrap_or_default();

    c.bench_function("sasspile_bootstrap_grid_67kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&input), sasspile::OutputStyle::Expanded));
        })
    });
}

fn bench_bootstrap_main(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap.scss")).unwrap_or_default();

    c.bench_function("sasspile_bootstrap_main_158kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(black_box(&input), sasspile::OutputStyle::Expanded));
        })
    });
}

criterion_group! {
    name = bootstrap_bench;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3)).sample_size(10);
    targets = bench_bootstrap_reboot, bench_bootstrap_grid, bench_bootstrap_main
}
criterion_main!(bootstrap_bench);
