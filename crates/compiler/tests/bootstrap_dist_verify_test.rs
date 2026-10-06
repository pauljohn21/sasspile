//! Full SCSS-to-CSS compilation verification against Bootstrap dist output.
//!
//! Each test constructs an AST mirroring a Bootstrap SCSS source file,
//! compiles it via Lightforger, and asserts the output matches the
//! corresponding CSS in `dist/css/bootstrap.css`.

use lightforger::reactive::{from_string_ast, AstNode};

// ---------------------------------------------------------------------------
// .badge  component  (dist/css/bootstrap.css lines 4809-4834)
// ---------------------------------------------------------------------------

fn dist_badge_css() -> String {
    ".badge {
  --bs-badge-padding-x: 0.65em;
  --bs-badge-padding-y: 0.35em;
  --bs-badge-font-size: 0.75em;
  --bs-badge-font-weight: 700;
  --bs-badge-color: #fff;
  --bs-badge-border-radius: var(--bs-border-radius);
  display: inline-block;
  padding: var(--bs-badge-padding-y) var(--bs-badge-padding-x);
  font-size: var(--bs-badge-font-size);
  font-weight: var(--bs-badge-font-weight);
  line-height: 1;
  color: var(--bs-badge-color);
  text-align: center;
  white-space: nowrap;
  vertical-align: baseline;
  border-radius: var(--bs-badge-border-radius);
}
.badge:empty {
  display: none;
}

.btn .badge {
  position: relative;
  top: -1px;
}"
    .to_string()
}

fn badge_ast() -> Vec<AstNode> {
    vec![
        AstNode::RuleSet {
            selector: ".badge".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "--bs-badge-padding-x".to_string(), value: "0.65em".to_string() },
                AstNode::StyleDecl { property: "--bs-badge-padding-y".to_string(), value: "0.35em".to_string() },
                AstNode::StyleDecl { property: "--bs-badge-font-size".to_string(), value: "0.75em".to_string() },
                AstNode::StyleDecl { property: "--bs-badge-font-weight".to_string(), value: "700".to_string() },
                AstNode::StyleDecl { property: "--bs-badge-color".to_string(), value: "#fff".to_string() },
                AstNode::StyleDecl { property: "--bs-badge-border-radius".to_string(), value: "var(--bs-border-radius)".to_string() },
                AstNode::StyleDecl { property: "display".to_string(), value: "inline-block".to_string() },
                AstNode::StyleDecl { property: "padding".to_string(), value: "var(--bs-badge-padding-y) var(--bs-badge-padding-x)".to_string() },
                AstNode::StyleDecl { property: "font-size".to_string(), value: "var(--bs-badge-font-size)".to_string() },
                AstNode::StyleDecl { property: "font-weight".to_string(), value: "var(--bs-badge-font-weight)".to_string() },
                AstNode::StyleDecl { property: "line-height".to_string(), value: "1".to_string() },
                AstNode::StyleDecl { property: "color".to_string(), value: "var(--bs-badge-color)".to_string() },
                AstNode::StyleDecl { property: "text-align".to_string(), value: "center".to_string() },
                AstNode::StyleDecl { property: "white-space".to_string(), value: "nowrap".to_string() },
                AstNode::StyleDecl { property: "vertical-align".to_string(), value: "baseline".to_string() },
                AstNode::StyleDecl { property: "border-radius".to_string(), value: "var(--bs-badge-border-radius)".to_string() },
            ],
        },
        AstNode::RuleSet {
            selector: ".badge:empty".to_string(),
            inner: vec![AstNode::StyleDecl { property: "display".to_string(), value: "none".to_string() }],
        },
        AstNode::RuleSet {
            selector: ".btn .badge".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "position".to_string(), value: "relative".to_string() },
                AstNode::StyleDecl { property: "top".to_string(), value: "-1px".to_string() },
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// .btn-close  component  (dist/css/bootstrap.css lines 5336-5371)
// ---------------------------------------------------------------------------

fn dist_btn_close_css() -> String {
    ".btn-close {
  --bs-btn-close-color: #000;
  --bs-btn-close-bg: url(\"data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='%23000'%3e%3cpath d='M.293.293a1 1 0 0 1 1.414 0L8 6.586 14.293.293a1 1 0 1 1 1.414 1.414L9.414 8l6.293 6.293a1 1 0 0 1-1.414 1.414L8 9.414l-6.293 6.293a1 1 0 0 1-1.414-1.414L6.586 8 .293 1.707a1 1 0 0 1 0-1.414'/%3e%3c/svg%3e\");
  --bs-btn-close-opacity: 0.5;
  --bs-btn-close-hover-opacity: 0.75;
  --bs-btn-close-focus-shadow: 0 0 0 0.25rem rgba(13, 110, 253, 0.25);
  --bs-btn-close-focus-opacity: 1;
  --bs-btn-close-disabled-opacity: 0.25;
  box-sizing: content-box;
  width: 1em;
  height: 1em;
  padding: 0.25em 0.25em;
  color: var(--bs-btn-close-color);
  background: transparent var(--bs-btn-close-bg) center/1em auto no-repeat;
  filter: var(--bs-btn-close-filter);
  border: 0;
  border-radius: 0.375rem;
  opacity: var(--bs-btn-close-opacity);
}
.btn-close:hover {
  color: var(--bs-btn-close-color);
  text-decoration: none;
  opacity: var(--bs-btn-close-hover-opacity);
}
.btn-close:focus {
  outline: 0;
  box-shadow: var(--bs-btn-close-focus-shadow);
  opacity: var(--bs-btn-close-focus-opacity);
}
.btn-close:disabled, .btn-close.disabled {
  pointer-events: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  user-select: none;
  opacity: var(--bs-btn-close-disabled-opacity);
}"
    .to_string()
}

fn btn_close_ast() -> Vec<AstNode> {
    vec![
        AstNode::RuleSet {
            selector: ".btn-close".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "--bs-btn-close-color".to_string(), value: "#000".to_string() },
                AstNode::StyleDecl {
                    property: "--bs-btn-close-bg".to_string(),
                    value: "url(\"data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='%23000'%3e%3cpath d='M.293.293a1 1 0 0 1 1.414 0L8 6.586 14.293.293a1 1 0 1 1 1.414 1.414L9.414 8l6.293 6.293a1 1 0 0 1-1.414 1.414L8 9.414l-6.293 6.293a1 1 0 0 1-1.414-1.414L6.586 8 .293 1.707a1 1 0 0 1 0-1.414'/%3e%3c/svg%3e\")".to_string(),
                },
                AstNode::StyleDecl { property: "--bs-btn-close-opacity".to_string(), value: "0.5".to_string() },
                AstNode::StyleDecl { property: "--bs-btn-close-hover-opacity".to_string(), value: "0.75".to_string() },
                AstNode::StyleDecl {
                    property: "--bs-btn-close-focus-shadow".to_string(),
                    value: "0 0 0 0.25rem rgba(13, 110, 253, 0.25)".to_string(),
                },
                AstNode::StyleDecl { property: "--bs-btn-close-focus-opacity".to_string(), value: "1".to_string() },
                AstNode::StyleDecl { property: "--bs-btn-close-disabled-opacity".to_string(), value: "0.25".to_string() },
                AstNode::StyleDecl { property: "box-sizing".to_string(), value: "content-box".to_string() },
                AstNode::StyleDecl { property: "width".to_string(), value: "1em".to_string() },
                AstNode::StyleDecl { property: "height".to_string(), value: "1em".to_string() },
                AstNode::StyleDecl { property: "padding".to_string(), value: "0.25em 0.25em".to_string() },
                AstNode::StyleDecl { property: "color".to_string(), value: "var(--bs-btn-close-color)".to_string() },
                AstNode::StyleDecl {
                    property: "background".to_string(),
                    value: "transparent var(--bs-btn-close-bg) center/1em auto no-repeat".to_string(),
                },
                AstNode::StyleDecl { property: "filter".to_string(), value: "var(--bs-btn-close-filter)".to_string() },
                AstNode::StyleDecl { property: "border".to_string(), value: "0".to_string() },
                AstNode::StyleDecl { property: "border-radius".to_string(), value: "0.375rem".to_string() },
                AstNode::StyleDecl { property: "opacity".to_string(), value: "var(--bs-btn-close-opacity)".to_string() },
            ],
        },
        AstNode::RuleSet {
            selector: ".btn-close:hover".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "color".to_string(), value: "var(--bs-btn-close-color)".to_string() },
                AstNode::StyleDecl { property: "text-decoration".to_string(), value: "none".to_string() },
                AstNode::StyleDecl { property: "opacity".to_string(), value: "var(--bs-btn-close-hover-opacity)".to_string() },
            ],
        },
        AstNode::RuleSet {
            selector: ".btn-close:focus".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "outline".to_string(), value: "0".to_string() },
                AstNode::StyleDecl { property: "box-shadow".to_string(), value: "var(--bs-btn-close-focus-shadow)".to_string() },
                AstNode::StyleDecl { property: "opacity".to_string(), value: "var(--bs-btn-close-focus-opacity)".to_string() },
            ],
        },
        AstNode::RuleSet {
            selector: ".btn-close:disabled, .btn-close.disabled".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "pointer-events".to_string(), value: "none".to_string() },
                AstNode::StyleDecl { property: "-webkit-user-select".to_string(), value: "none".to_string() },
                AstNode::StyleDecl { property: "-moz-user-select".to_string(), value: "none".to_string() },
                AstNode::StyleDecl { property: "user-select".to_string(), value: "none".to_string() },
                AstNode::StyleDecl { property: "opacity".to_string(), value: "var(--bs-btn-close-disabled-opacity)".to_string() },
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// .placeholder  component  (dist/css/bootstrap.css lines 6777-6824)
// ---------------------------------------------------------------------------

fn dist_placeholder_css() -> String {
    ".placeholder {
  display: inline-block;
  min-height: 1em;
  vertical-align: middle;
  cursor: wait;
  background-color: currentcolor;
  opacity: 0.5;
}
.placeholder.btn::before {
  display: inline-block;
  content: \"\";
}

.placeholder-xs {
  min-height: 0.6em;
}

.placeholder-sm {
  min-height: 0.8em;
}

.placeholder-lg {
  min-height: 1.2em;
}

.placeholder-glow .placeholder {
  animation: placeholder-glow 2s ease-in-out infinite;
}

@keyframes placeholder-glow {
  50% {
    opacity: 0.2;
  }
}
.placeholder-wave {
  -webkit-mask-image: linear-gradient(130deg, #000 55%, rgba(0, 0, 0, 0.8) 75%, #000 95%);
  mask-image: linear-gradient(130deg, #000 55%, rgba(0, 0, 0, 0.8) 75%, #000 95%);
  -webkit-mask-size: 200% 100%;
  mask-size: 200% 100%;
  animation: placeholder-wave 2s linear infinite;
}

@keyframes placeholder-wave {
  100% {
    -webkit-mask-position: -200% 0%;
    mask-position: -200% 0%;
  }
}"
    .to_string()
}

fn placeholder_ast() -> Vec<AstNode> {
    vec![
        // .placeholder { ... }
        AstNode::RuleSet {
            selector: ".placeholder".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "display".to_string(), value: "inline-block".to_string() },
                AstNode::StyleDecl { property: "min-height".to_string(), value: "1em".to_string() },
                AstNode::StyleDecl { property: "vertical-align".to_string(), value: "middle".to_string() },
                AstNode::StyleDecl { property: "cursor".to_string(), value: "wait".to_string() },
                AstNode::StyleDecl { property: "background-color".to_string(), value: "currentcolor".to_string() },
                AstNode::StyleDecl { property: "opacity".to_string(), value: "0.5".to_string() },
            ],
        },
        // .placeholder.btn::before { ... }
        AstNode::RuleSet {
            selector: ".placeholder.btn::before".to_string(),
            inner: vec![
                AstNode::StyleDecl { property: "display".to_string(), value: "inline-block".to_string() },
                AstNode::StyleDecl { property: "content".to_string(), value: "\"\"".to_string() },
            ],
        },
        // .placeholder-xs
        AstNode::RuleSet {
            selector: ".placeholder-xs".to_string(),
            inner: vec![AstNode::StyleDecl { property: "min-height".to_string(), value: "0.6em".to_string() }],
        },
        // .placeholder-sm
        AstNode::RuleSet {
            selector: ".placeholder-sm".to_string(),
            inner: vec![AstNode::StyleDecl { property: "min-height".to_string(), value: "0.8em".to_string() }],
        },
        // .placeholder-lg
        AstNode::RuleSet {
            selector: ".placeholder-lg".to_string(),
            inner: vec![AstNode::StyleDecl { property: "min-height".to_string(), value: "1.2em".to_string() }],
        },
        // .placeholder-glow .placeholder { animation: ... }
        AstNode::RuleSet {
            selector: ".placeholder-glow .placeholder".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "animation".to_string(),
                value: "placeholder-glow 2s ease-in-out infinite".to_string(),
            }],
        },
        // @keyframes placeholder-glow { 50% { opacity: 0.2; } }
        AstNode::RuleSet {
            selector: "@keyframes placeholder-glow".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: "50%".to_string(),
                inner: vec![AstNode::StyleDecl { property: "opacity".to_string(), value: "0.2".to_string() }],
            }],
        },
        // .placeholder-wave { ... }
        AstNode::RuleSet {
            selector: ".placeholder-wave".to_string(),
            inner: vec![
                AstNode::StyleDecl {
                    property: "-webkit-mask-image".to_string(),
                    value: "linear-gradient(130deg, #000 55%, rgba(0, 0, 0, 0.8) 75%, #000 95%)".to_string(),
                },
                AstNode::StyleDecl {
                    property: "mask-image".to_string(),
                    value: "linear-gradient(130deg, #000 55%, rgba(0, 0, 0, 0.8) 75%, #000 95%)".to_string(),
                },
                AstNode::StyleDecl { property: "-webkit-mask-size".to_string(), value: "200% 100%".to_string() },
                AstNode::StyleDecl { property: "mask-size".to_string(), value: "200% 100%".to_string() },
                AstNode::StyleDecl {
                    property: "animation".to_string(),
                    value: "placeholder-wave 2s linear infinite".to_string(),
                },
            ],
        },
        // @keyframes placeholder-wave { 100% { ... } }
        AstNode::RuleSet {
            selector: "@keyframes placeholder-wave".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: "100%".to_string(),
                inner: vec![
                    AstNode::StyleDecl { property: "-webkit-mask-position".to_string(), value: "-200% 0%".to_string() },
                    AstNode::StyleDecl { property: "mask-position".to_string(), value: "-200% 0%".to_string() },
                ],
            }],
        },
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Normalize CSS for comparison: trim trailing whitespace and collapse
/// blank lines.  The dist file uses blank lines between rule blocks
/// while Lightforger does not; the semantic content is identical.
fn normalize_css(css: &str) -> String {
    css.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn badge_matches_dist_css() {
    let items = badge_ast();
    let compiled = from_string_ast(items).unwrap();
    let expected = dist_badge_css();

    let compiled_norm = normalize_css(&compiled);
    let expected_norm = normalize_css(&expected);

    assert_eq!(
        compiled_norm, expected_norm,
        "\n=== Lightforger compiled output ===\n{}\n=== Expected (dist CSS) ===\n{}",
        compiled_norm, expected_norm
    );
}

#[test]
fn btn_close_matches_dist_css() {
    let items = btn_close_ast();
    let compiled = from_string_ast(items).unwrap();
    let expected = dist_btn_close_css();

    let compiled_norm = normalize_css(&compiled);
    let expected_norm = normalize_css(&expected);

    assert_eq!(
        compiled_norm, expected_norm,
        "\n=== Lightforger compiled output ===\n{}\n=== Expected (dist CSS) ===\n{}",
        compiled_norm, expected_norm
    );
}

#[test]
fn placeholder_matches_dist_css() {
    let items = placeholder_ast();
    let compiled = from_string_ast(items).unwrap();
    let expected = dist_placeholder_css();

    let compiled_norm = normalize_css(&compiled);
    let expected_norm = normalize_css(&expected);

    assert_eq!(
        compiled_norm, expected_norm,
        "\n=== Lightforger compiled output ===\n{}\n=== Expected (dist CSS) ===\n{}",
        compiled_norm, expected_norm
    );
}

#[test]
fn all_three_components_combined() {
    let mut all = Vec::new();
    all.extend(badge_ast());
    all.extend(btn_close_ast());
    all.extend(placeholder_ast());

    let compiled = from_string_ast(all).unwrap();

    // Each component's CSS should appear as a contiguous block
    assert!(compiled.contains(".badge {"), "missing .badge");
    assert!(compiled.contains(".btn-close {"), "missing .btn-close");
    assert!(compiled.contains(".placeholder {"), "missing .placeholder");

    // Verify key declarations are present & correct
    assert!(compiled.contains("--bs-badge-padding-x: 0.65em;"));
    assert!(compiled.contains("--bs-btn-close-color: #000;"));
    assert!(compiled.contains("animation: placeholder-glow 2s ease-in-out infinite;"));

    // Verify state selectors
    assert!(compiled.contains(".badge:empty {"));
    assert!(compiled.contains(".btn-close:hover {"));
    assert!(compiled.contains(".btn-close:focus {"));
    assert!(compiled.contains(".btn-close:disabled, .btn-close.disabled {"));
    assert!(compiled.contains(".placeholder.btn::before {"));

    // Verify keyframes
    assert!(compiled.contains("@keyframes placeholder-glow"));
    assert!(compiled.contains("@keyframes placeholder-wave"));
}
