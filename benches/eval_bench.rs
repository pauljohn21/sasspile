use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sasspile::{OutputStyle, compile};

fn sample_scss() -> String {
    r#"
// Variables
$primary-color: #3498db;
$secondary-color: #2ecc71;
$font-stack: 'Helvetica Neue', Helvetica, Arial, sans-serif;
$base-padding: 16px;

// Base styles
body {
    font-family: $font-stack;
    color: $primary-color;
    padding: $base-padding;
}

// Navigation
nav {
    background-color: $primary-color;
    padding: $base-padding;

    ul {
        list-style: none;
        margin: 0;
        padding: 0;

        li {
            display: inline-block;
            margin-right: $base-padding;

            a {
                color: white;
                text-decoration: none;

                &:hover {
                    text-decoration: underline;
                }
            }
        }
    }
}

// Buttons
.btn {
    display: inline-block;
    padding: 10px 20px;
    border: none;
    cursor: pointer;
}

.btn-primary {
    @extend .btn;
    background-color: $primary-color;
    color: white;
}
"#
    .repeat(5)
}

fn bench_eval_expanded(c: &mut Criterion) {
    let input = sample_scss();
    c.bench_function("eval_expanded_7kb", |b| {
        b.iter(|| {
            let _ = black_box(compile(black_box(&input), OutputStyle::Expanded));
        })
    });
}

fn bench_eval_compressed(c: &mut Criterion) {
    let input = sample_scss();
    c.bench_function("eval_compressed_7kb", |b| {
        b.iter(|| {
            let _ = black_box(compile(black_box(&input), OutputStyle::Compressed));
        })
    });
}

criterion_group!(benches, bench_eval_expanded, bench_eval_compressed);
criterion_main!(benches);
