use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sasspile::{Lexer, Parser};

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

fn bench_parser(c: &mut Criterion) {
    let input = sample_scss();
    c.bench_function("parser_14kb_scss", |b| {
        b.iter(|| {
            let tokens: Vec<_> = Lexer::new(black_box(&input))
                .filter(|t| {
                    !matches!(
                        t.as_ref(),
                        Ok(sasspile::lex::token::Token::Whitespace)
                            | Ok(sasspile::lex::token::Token::Eof)
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let ast = Parser::parse(&tokens);
            let _ = black_box(ast);
        })
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
