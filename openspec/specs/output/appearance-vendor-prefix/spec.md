# AppearanceVendorPrefix Specification

## Purpose

Defines that the CSS serializer MUST emit `-moz-appearance: none` before `appearance: none` to match dart-sass output for full browser compatibility.

## Requirements

### Requirement: appearance outputs -moz- appearance prefix

When serializing the CSS property `appearance`, the serializer SHALL emit `-moz-appearance: none` immediately before `appearance: none`.

#### Scenario: appearance: none in button or input mixin
- **WHEN** the SCSS source contains `appearance: none` (typically in form control reset mixins)
- **THEN** the output SHALL contain `-moz-appearance: none; appearance: none`
- **AND** the `-moz-` variant MUST appear first

#### Scenario: appearance with non-none value
- **WHEN** the SCSS source contains `appearance: textfield`
- **THEN** the output SHALL be `appearance: textfield` (no vendor prefix needed)

#### Scenario: -moz-appearance already present
- **WHEN** the SCSS source explicitly declares `-moz-appearance: none`
- **THEN** the output SHALL retain it without duplication
