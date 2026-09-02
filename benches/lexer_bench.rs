use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sasspile::Lexer;

fn sample_scss() -> String {
    r#"
// Variables
$primary-color: #3498db;
$secondary-color: #2ecc71;
$font-stack: 'Helvetica Neue', Helvetica, Arial, sans-serif;
$base-padding: 16px;

// Mixins
@mixin border-radius($radius) {
    -webkit-border-radius: $radius;
    -moz-border-radius: $radius;
    border-radius: $radius;
}

@mixin box-shadow($x, $y, $blur, $color) {
    -webkit-box-shadow: $x $y $blur $color;
    -moz-box-shadow: $x $y $blur $color;
    box-shadow: $x $y $blur $color;
}

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
    @include border-radius(4px);
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
    .repeat(10)
}

fn bench_lexer(c: &mut Criterion) {
    let input = sample_scss();
    c.bench_function("lexer_14kb_scss", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(&input));
            let tokens: Vec<_> = lexer.collect();
            black_box(tokens);
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
