//! rxrust 管线 vs 同步管线 性能对比
//!
//! 运行：cargo bench --bench rxrust_compare

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use std::time::Duration;

fn bs_scss(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bs")
        .join("scss")
        .join(file)
}

// ===== 同步管线（原始）=====

fn bench_sync_bootstrap_reboot(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap-reboot.scss")).unwrap_or_default();

    c.bench_function("sync_bootstrap_reboot_48kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(
                black_box(&input),
                sasspile::OutputStyle::Expanded,
            ));
        })
    });
}

fn bench_sync_bootstrap_main(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap.scss")).unwrap_or_default();

    c.bench_function("sync_bootstrap_main_158kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile(
                black_box(&input),
                sasspile::OutputStyle::Expanded,
            ));
        })
    });
}

// ===== rxrust 管线 =====

fn bench_rxrust_bootstrap_reboot(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap-reboot.scss")).unwrap_or_default();

    c.bench_function("rxrust_bootstrap_reboot_48kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile_rx::compile_rx(
                black_box(&input),
                sasspile::OutputStyle::Expanded,
            ));
        })
    });
}

fn bench_rxrust_bootstrap_main(c: &mut Criterion) {
    let input = std::fs::read_to_string(bs_scss("bootstrap.scss")).unwrap_or_default();

    c.bench_function("rxrust_bootstrap_main_158kb", |b| {
        b.iter(|| {
            let _ = black_box(sasspile::compile_rx::compile_rx(
                black_box(&input),
                sasspile::OutputStyle::Expanded,
            ));
        })
    });
}

criterion_group! {
    name = rxrust_compare;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(10);
    targets = bench_sync_bootstrap_reboot,
             bench_sync_bootstrap_main,
             bench_rxrust_bootstrap_reboot,
             bench_rxrust_bootstrap_main
}
criterion_main!(rxrust_compare);
