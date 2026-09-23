## Capability: css-serializer

### ADDED Requirements

#### Requirement: CssNode → CSS string rendering
`render_node(&CssNode) -> String` SHALL produce formatted CSS string.

#### Scenario: Rule rendering
- **WHEN** CssNode::Rule { selector: ".foo", children: [Declaration("color", "red")] }
- **THEN** output = ".foo {\n  color: red;\n}"

#### Scenario: Nested rule rendering
- **WHEN** Rule(".parent", [Rule(".child", [...])])
- **THEN** output ".parent {\n  .child {\n    ...\n  }\n}" (nested, not flattened)

#### Scenario: AtRule media query
- **WHEN** AtRule("@media (max-width: 768px)", [...])
- **THEN** output ".media-query {\n  ...\n}"

#### Scenario: AtRoot passthrough
- **WHEN** AtRoot([Rule(".bar", [...])])
- **THEN** output renders inner Rule directly without AtRoot wrapper

---

### Requirement: Integration with pipeline
Final compilation chain SHALL include AST → String conversion as last stage.

#### Scenario: Pipeline output contract
```
css_node_stream
  .flat_map(|node| from_iter(render_node_chars(node)))
  .collect::<String>()
  .last()
  .subscribe(|css| tx.send(css));
```
