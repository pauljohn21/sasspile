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

.btn-secondary {
    @extend .btn;
    background-color: $secondary-color;
    color: white;
}

// Media queries
@media (max-width: 768px) {
    nav ul li {
        display: block;
        margin-bottom: $base-padding / 2;
    }

    .btn {
        width: 100%;
    }
}
"#
    .repeat(20)
}

fn bench_compile_full(c: &mut Criterion) {
    let input = sample_scss();
    c.bench_function("compile_full_28kb_expanded", |b| {
        b.iter(|| {
            let _ = black_box(compile(black_box(&input), OutputStyle::Expanded));
        })
    });
}

criterion_group!(benches, bench_compile_full);
criterion_main!(benches);
