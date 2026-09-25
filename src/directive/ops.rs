//! 独立 block 展开 — flat_map reducer (无 zxrust trait, 纯函数)
//!
//! DirectiveBlock → Vec<String> (借 &mut CompileState 就地修改 + &CompileState 读取)

use super::blocks::DirectiveBlock;
use super::parse::{expand_mixin, parse_include_sig, substitute_vars};
use super::state::CompileState;

/// 跨行 @extend 合并预处理
/// "d {@extend" + "a}" → "d {@extend a}"
/// "d {@extend" + "a" + "}" → "d {@extend a}"
/// "a {@extend b" + " !optional}" → "a {@extend b !optional}"
/// "a {@extend b" + " !optional" + "}" → "a {@extend b !optional}"
fn merge_extend_continuations(lines: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();
        // 检测: 行内有 @extend 但 extend 不完整 (缺少目标就在本行)
        // 即: @extend 后只有空白或换行,@extend 是行尾最后一个 token
        let ends_with_extend = trimmed.ends_with("@extend") || trimmed.ends_with("@extend ");
        // 或者行内有 @extend 但 } 在后续行
        let has_incomplete_extend = trimmed.contains("@extend")
            && !trimmed.contains('}')
            && (trimmed.ends_with("@extend") || trimmed.trim_end().ends_with("@extend"));

        if ends_with_extend || has_incomplete_extend {
            // 收集后续行直到 } 出现
            let mut combined = trimmed.to_string();
            let mut j = i + 1;
            let mut found_close = false;
            while j < lines.len() {
                let next = lines[j].trim();
                if next == "}" {
                    found_close = true;
                    // 不追加空 }, 因为是规则结束
                    j += 1;
                    break;
                } else if next.starts_with("}") {
                    // "} ..." or " !optional}" — extract content before }
                    let before_close = next.trim_start_matches('}').trim();
                    if !before_close.is_empty() {
                        combined.push(' ');
                        combined.push_str(before_close);
                    }
                    found_close = true;
                    j += 1;
                    break;
                } else if next.trim_end_matches('}').trim_end() != next {
                    // contains } but not at start: " !optional}" or "a}"
                    let before_close = next.trim_end_matches('}').trim();
                    if !before_close.is_empty() {
                        combined.push(' ');
                        combined.push_str(before_close);
                    }
                    found_close = true;
                    j += 1;
                    break;
                } else {
                    combined.push(' ');
                    combined.push_str(next);
                }
                j += 1;
            }
            if found_close || !combined.ends_with("@extend") {
                result.push(combined);
                i = j;
                continue;
            }
        }
        result.push(line.clone());
        i += 1;
    }
    result
}

pub fn process_block(block: DirectiveBlock, state: &mut CompileState) -> Vec<String> {
    let state_ref: &CompileState = &*state;

    match block {
        DirectiveBlock::Lines(lines) => {
            // 预处理: 合并跨行 @extend ("d {@extend" + "a}" → "d {@extend a}")
            let merged = merge_extend_continuations(&lines);
            merged.into_iter().flat_map(|line| process_line(&line, state)).collect()
        }
        DirectiveBlock::For { ref var_name, ref values, ref body } => values
            .iter()
            .flat_map(|v| body.iter().map(|b| {
                // 先替换 #{$var} 整体, 再替换裸 $var, 避免破坏插值语法
                let step1 = b.replace(&format!("#{{{var_name}}}"), v);
                substitute_vars(state_ref, &step1.replace(var_name, v))
            }))
            .collect(),
        DirectiveBlock::Each { ref var_name, ref items, ref body } => items
            .iter()
            .flat_map(|i| body.iter().map(|b| {
                let step1 = b.replace(&format!("#{{{var_name}}}"), i);
                substitute_vars(state_ref, &step1.replace(var_name, i))
            }))
            .collect(),
        DirectiveBlock::If { ref branches } => {
            for (cond, body) in branches {
                let take = match cond {
                    None => true,
                    Some(expr) => eval_condition(state_ref, expr),
                };
                if take { return body.clone(); }
            }
            vec![]
        }
        DirectiveBlock::MixinDef { name, params, body } => {
            state.scope.mixins.insert(name, crate::directive::state::MixinDef { params, body });
            vec![]
        }
        DirectiveBlock::PlaceholderDef { name, body } => {
            state.placeholder_defs.insert(name, body);
            vec![]
        }
        DirectiveBlock::While { cond, body } => expand_while(state, &cond, &body),
        DirectiveBlock::Include { name, args, using, body } => {
            expand_include_multi(state, &name, &args, &using, &body)
        }
    }
}

/// 展开多行 @include (含 using 子句 + content 块)
///
/// content block 的 body 行作为独立 CSS 行 emit,不做 join — 保留原有结构,
/// CssBuilder 能正确解析嵌套规则
fn expand_include_multi(state: &CompileState, name: &str, args: &[String], using: &[String], body: &[String]) -> Vec<String> {
    let Some(def) = state.scope.mixins.get(name) else {
        return vec![];
    };

    let mut effective_args = args.to_vec();
    if !using.is_empty() {
        effective_args.extend_from_slice(using);
    }

    let expanded = expand_mixin(def, &effective_args);

    // 展开: 逐行扫描 mixin body, 遇到 @content 则用 body 行替代, 否则保留该 line
    expanded.iter().flat_map(|line| {
        if line.contains("@content") {
            // 用 content body 行替代 @content 占位符 (保留结构, 非 join)
            body.iter().map(|b| b.as_str()).collect::<Vec<&str>>()
        } else {
            vec![line.as_str()]
        }
    }).map(|s| s.to_string())  // &str → String (flat_map 需要 owned flat_map)
    .collect()
}

/// @while 展开: 求值 cond, 若 true 则展开 body, 跳过 $@var 赋值, 直到 false 或 body 耗尽Safety: body 中必须包含变量mutation (如 `$i: $i + 1;`) 以防无限循环
fn expand_while(state: &CompileState, cond: &str, body: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let max_iter = 100; // 防止无限循环的安全上限
    let mut current_state = state.clone();

    for _ in 0..max_iter {
        // 求值条件
        let cond_substituted = crate::directive::parse::substitute_vars(&current_state, cond);
        if !eval_while_condition(&cond_substituted) {
            break;
        }

        // 展开 body: 处理变量赋值和 CSS 行
        for line in body {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            // 变量赋值: "$i: $i + 1;" → 更新 state
            if trimmed.starts_with('$') && trimmed.contains(':') {
                if let Some((name, value)) = parse_var_assignment(trimmed) {
                    let evaluated = eval_expr_simple(&current_state, &value);
                    current_state.scope.variables.insert(name, evaluated);
                }
                continue;
            }

            // CSS 行: 替换变量后 emit
            let substituted = crate::directive::parse::substitute_vars(&current_state, trimmed);
            result.push(substituted);
        }
    }

    result
}

/// 简单表达式求值: "$i + 1" / "$i - 1" 等
fn eval_expr_simple(state: &CompileState, expr: &str) -> String {
    let substituted = crate::directive::parse::substitute_vars(state, expr);
    let t = substituted.trim();

    // 尝试简单算术: "$i + 1" / "5 + 1"
    if let Some(idx) = t.find('+') {
        let (l, r) = t.split_at(idx);
        if let (Ok(a), Ok(b)) = (l.trim().parse::<f64>(), r[1..].trim().parse::<f64>()) {
            let sum = a + b;
            return if sum.fract() == 0.0 { format!("{:.0}", sum) } else { sum.to_string() };
        }
    }
    if let Some(idx) = t.find('-') {
        let (l, r) = t.split_at(idx);
        if let (Ok(a), Ok(b)) = (l.trim().parse::<f64>(), r[1..].trim().parse::<f64>()) {
            let diff = a - b;
            return if diff.fract() == 0.0 { format!("{:.0}", diff) } else { diff.to_string() };
        }
    }
    t.to_string()
}

/// 解析变量赋值: "$i: $i + 1;" → ("$i", "$i + 1")
fn parse_var_assignment(line: &str) -> Option<(String, String)> {
    if !line.starts_with('$') { return None; }
    let (name, value_part) = line[1..].split_once(':')?;
    let name = format!("${}", name.trim());
    let value = value_part.trim().trim_end_matches(';').trim().to_string();
    if value.is_empty() { return None; }
    Some((name, value))
}

/// @while 条件求值
fn eval_while_condition(cond: &str) -> bool {
    let t = cond.trim().replace("#{", "").replace('}', "");

    match t.as_str() {
        "false" | "null" | "0" | "" => return false,
        "true" => return true,
        _ => {}
    }

    // 按优先级尝试操作符 (双字符优先)
    if let Some(r) = try_compare(&t, "<=") { return r; }
    if let Some(r) = try_compare(&t, ">=") { return r; }
    if let Some(r) = try_compare(&t, "==") { return r; }
    if let Some(r) = try_compare(&t, "!=") { return r; }
    if let Some(r) = try_compare(&t, "<") { return r; }
    if let Some(r) = try_compare(&t, ">") { return r; }

    true
}

fn try_compare(t: &str, op: &str) -> Option<bool> {
    let idx = t.find(op)?;
    let (l, r) = t.split_at(idx);
    let r = &r[op.len()..];
    let l_val: f64 = l.trim().parse().ok()?;
    let r_val: f64 = r.trim().parse().ok()?;
    Some(match op {
        "<=" => l_val <= r_val,
        ">=" => l_val >= r_val,
        "<" => l_val < r_val,
        ">" => l_val > r_val,
        "==" => (l_val - r_val).abs() < f64::EPSILON,
        "!=" => (l_val - r_val).abs() > f64::EPSILON,
        _ => return None,
    })
}

/// 单行处理: 变量 def / @include / @extend @ plain CSS
/// 同时维护跨行规则上下文: .selector { ... @extend ... }
fn process_line(line: &str, state: &mut CompileState) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    // 规则关闭: 先注入所有 pending @extend declarations
    if trimmed == "}" && state.current_rule_name.is_some() {
        let mut out = Vec::new();
        // 注入 pending extends
        if !state.pending_extend_decls.is_empty() {
            for d in state.pending_extend_decls.drain(..) {
                out.push(format!("  {d};"));
            }
        }
        out.push(line.to_string());
        state.current_rule_name = None;
        return out;
    }

    // 嵌套子规则 (规则体内) 也由上面的 "}" 处理

    if trimmed.starts_with('$') && trimmed.contains(':') {
        if let Some((name, value)) = parse_var_def(trimmed) {
            state.scope.variables.insert(name, value);
        }
        return vec![];
    }
    if trimmed.starts_with("@include ") {
        return handle_include(trimmed, state);
    }
    if trimmed.starts_with("@extend ") && state.current_rule_name.is_some() {
        // 跨行规则体中的 @extend: 尝试 placeholder 注入
        if let Some(decl) = parse_inline_extend(trimmed, state) {
            state.pending_extend_decls.push(decl);
            return vec![];
        }
        // 选择器级 @extend: 输出 ExtendMarker 到管线
        if let Some(marker) = build_extend_marker(trimmed, state) {
            return vec![marker];
        }
        return vec![];
    }
    // 行内 @extend: ".bar { @extend %foo; }" → 注入 placeholder decls
    if trimmed.contains("@extend ") && trimmed.ends_with('}') {
        // 提取 extend target (判断是否为 placeholder)
        let after_extend = trimmed.split("@extend ").nth(1).unwrap_or("");
        let target_spec = after_extend
            .split(|c: char| c == ';' || c == '}')
            .next()
            .unwrap_or("")
            .trim();
        let is_placeholder = target_spec.starts_with('%');
        let placeholder_name = target_spec.strip_prefix('%')
            .map(|s| s.trim_end_matches("!optional").trim())
            .unwrap_or("");
        let is_optional = target_spec.contains("!optional");
        let placeholder_exists = state.placeholder_defs.contains_key(placeholder_name);

        if is_placeholder && placeholder_exists {
            // placeholder 存在: 注入 declarations
            let placeholder_result = handle_inline_extend(trimmed, state);
            if !placeholder_result.is_empty() {
                return placeholder_result;
            }
        }

        // 选择器级 @extend: 输出 ExtendMarker
        if let Some(marker) = build_extend_marker(trimmed, state) {
            // 判断规则体是否只包含 @extend (无其他声明)
            let inner = trimmed
                .trim_start_matches(|c: char| c != '{')
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            // 规则体只含 @extend: 去掉末尾 ; 后就是 "@extend ..." 且声明内部没有额外 ;
            let only_extend = {
                let without_semi = inner.trim_end_matches(';').trim();
                without_semi.starts_with("@extend")
                    && !without_semi[7..].contains(';')
            };

            if is_placeholder && is_optional && !placeholder_exists {
                // !optional + placeholder 不存在: 移除 @extend 行, 保留其余声明
                if only_extend {
                    // 规则体只有 @extend: 不输出空规则
                    return vec![];
                }
                let without_extend = remove_extend_line(trimmed);
                if without_extend.is_empty() || without_extend == "{" || without_extend == "{}" {
                    return vec![];
                }
                return vec![without_extend];
            }

            if only_extend {
                return vec![marker];
            }
            // 规则体有其他声明: 同时输出 marker + 原规则 (原规则去掉 @extend 行)
            let without_extend = remove_extend_line(trimmed);
            if without_extend.is_empty() || without_extend == "{" || without_extend == "{}" {
                return vec![marker];
            }
            return vec![marker, without_extend];
        }

        // extend 解析失败: 回退到原规则输出
        return vec![substitute_vars(&*state, trimmed)];
    }

    // 规则开启: ".wrapper {"
    if !trimmed.starts_with('@') && !trimmed.starts_with('$') && trimmed.ends_with('{') {
        if let Some(selector) = trimmed.strip_suffix('{').map(str::trim) {
            if !selector.is_empty() && !selector.starts_with('@') {
                state.current_rule_name = Some(selector.to_string());
            }
        }
        return vec![substitute_vars(&*state, trimmed)];
    }

    vec![substitute_vars(&*state, trimmed)]
}

/// 构建 extend 标记字符串 (注入管线 → CssBuilder → CssNode::ExtendMarker)
/// line 必须是原始规则行 (如 "d {@extend a}" 或 "@extend a")
fn build_extend_marker(line: &str, state: &CompileState) -> Option<String> {
    // 确定 extender
    let extender = if let Some(name) = &state.current_rule_name {
        name.clone()
    } else {
        // 单行规则: 从 "{ " 前提取选择器
        line.split('{').next()?.trim().to_string()
    };
    if extender.is_empty() {
        return None;
    }
    // 提取 target: 从 "@extend " 后开始
    let after_extend = line.split("@extend ").nth(1)?;
    // 剥离后续内容 (;, }, 行尾)
    let target_spec = after_extend
        .split(|c: char| c == ';' || c == '}')
        .next()?
        .trim();
    if target_spec.is_empty() {
        return None;
    }
    // 剥离行末注释
    let target_spec = if let Some(idx) = target_spec.find("//") {
        &target_spec[..idx]
    } else {
        target_spec
    };
    // 剥离块注释
    let target_spec = target_spec.replace("/**/", "").replace("/*", "").replace("*/", "");
    let target_spec = target_spec.trim();
    if target_spec.is_empty() {
        return None;
    }
    // 检查 !optional (支持多目标时只检查整体)
    let optional = target_spec.contains("!optional");
    // 清理每个 target 的 !optional
    let targets: Vec<String> = target_spec
        .split(',')
        .map(|t| t.trim().trim_end_matches("!optional").trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if targets.is_empty() {
        return None;
    }
    // 如果 extender 在 targets 中, 跳过所有 (自扩展)
    if targets.iter().any(|t| t == &extender) {
        return None;
    }
    // 用冒号连接多目标
    let target = targets.join(":");
    Some(format!(">>EXTEND:{}:{}:{}", extender, target, optional))
}

/// 从单行 @extend 提取 placeholder declaration (跨行/单行通用)
fn parse_inline_extend(line: &str, state: &CompileState) -> Option<String> {
    let after = if let Some(extend_idx) = line.find("@extend ") {
        &line[extend_idx + 8..]
    } else {
        line
    };
    let end_rel = after.find(|c: char| c == ';' || c == '}')?;
    let placeholder = after[..end_rel].trim().trim_end_matches(';').trim();
    let placeholder = placeholder.strip_suffix("!optional").unwrap_or(placeholder).trim();
    let placeholder_name = placeholder.strip_prefix('%').unwrap_or(placeholder);

    let body = state.placeholder_defs.get(placeholder_name)?;
    Some(extract_declarations(body).trim().to_string())
}

/// 处理行内 @extend: ".bar { @extend %foo; }" → ".bar { color: red; }"
fn handle_inline_extend(line: &str, state: &CompileState) -> Vec<String> {
    let Some(extend_start) = line.find("@extend ") else {
        return vec![substitute_vars(state, line)];
    };
    let after_extend = &line[extend_start + 8..];
    let Some(semi_rel) = after_extend.find(|c: char| c == ';' || c == '}') else {
        return vec![substitute_vars(state, line)];
    };
    let placeholder_name = after_extend[..semi_rel].trim().trim_end_matches(';').trim();
    let (placeholder_name, _optional) = match placeholder_name.strip_suffix("!optional") {
        Some(name) => (name.trim(), true),
        None => (placeholder_name, false),
    };
    // 去掉可能的 % 前缀
    let placeholder_name = placeholder_name.strip_prefix('%').unwrap_or(placeholder_name);

    // 获取 placeholder body (declarations 行)
    let body = state.placeholder_defs.get(placeholder_name);

    // 构造移除 @extend 后的行
    let without_extend_slice = format!("{}{}", &line[..extend_start], &line[extend_start + 8 + semi_rel + 1..]);
    let without_extend = without_extend_slice.trim();

    let Some(body) = body else {
        // placeholder 不存在: 保留空移除 @extend
        if without_extend.is_empty() || without_extend == "}" || without_extend == "{ }" || without_extend == "{}" {
            return vec![];
        }
        return vec![substitute_vars(state, without_extend)];
    };

    // 从 placeholder body 中提取 declarations (去掉外层花括号)
    let decls = extract_declarations(body);

    // 找到 { 和 } 的位置来插入 declarations
    let Some(open_brace) = without_extend.rfind('{') else {
        return vec![decls];
    };
    let Some(close_brace) = without_extend.rfind('}') else {
        return vec![decls];
    };

    let inner = format!("{}{}{}", &without_extend[..open_brace + 1], decls, &without_extend[close_brace..]);
    vec![substitute_vars(state, &inner)]
}

/// 从 placeholder body 提取 declarations (去掉外层花括号)
fn extract_declarations(body: &[String]) -> String {
    let joined = body.join(" ");
    // 去掉最外层的 { }
    let trimmed = joined.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn handle_include(line: &str, state: &CompileState) -> Vec<String> {
    let (name, args) = parse_include_sig(&line[9..]);
    state
        .scope
        .mixins
        .get(&name)
        .map(|def| {
            expand_mixin(def, &args)
                .into_iter()
                .flat_map(|b| expand_single_line(&b, state))
                .collect()
        })
        .unwrap_or_default()
}


fn expand_single_line(line: &str, state: &CompileState) -> Vec<String> {
    let tr = line.trim();
    if let (true, true) = (tr.starts_with('{'), tr.ends_with('}')) {
        let inner = &tr[1..tr.len() - 1];
        inner
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("{};", substitute_vars(state, s)))
            .collect()
    } else {
        vec![substitute_vars(state, tr)]
    }
}

fn parse_var_def(line: &str) -> Option<(String, String)> {
    if !line.starts_with('$') {
        return None;
    }
    let after_dollar = &line[1..];
    let (name, value_part) = after_dollar.split_once(':')?;
    let name = format!("${}", name.trim());
    let value = value_part.trim().trim_end_matches(';').trim().to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value))
}

/// 从规则行中移除 @extend 行 (保留其余声明)
fn remove_extend_line(line: &str) -> String {
    let Some(extend_rel) = line.find("@extend ") else {
        return line.to_string();
    };
    let after_extend = &line[extend_rel + 8..];
    let end_rel = after_extend.find(|c: char| c == ';' || c == '}').unwrap_or(after_extend.len());
    // 构造: @extend 之前的部分 + @extend target 之后的部分
    let before = &line[..extend_rel];
    let after = &line[extend_rel + 8 + end_rel..];
    // 跳过后面的 ; 或 }
    let after = after.strip_prefix(';').unwrap_or(after);
    let result = format!("{before}{after}");
    let result = result.trim();
    // 如果结果是 ".selector { }" 形式，只保留 ".selector { ... }"
    if let Some(open) = result.find('{') {
        if let Some(close) = result.rfind('}') {
            let inner = &result[open + 1..close].trim();
            if inner.is_empty() {
                return String::new();
            }
        }
    }
    result.to_string()
}

fn eval_condition(state: &CompileState, expr: &str) -> bool {
    let expr = substitute_vars(state, expr);
    let t = expr.trim();
    if t.is_empty() || t == "false" || t == "null" || t == "0" {
        return false;
    }
    if t == "true" {
        return true;
    }
    if let Some(idx) = t.find("==") {
        let (l, r) = t.split_at(idx);
        return l.trim() == r[2..].trim().trim_matches('"');
    }
    if let Some(idx) = t.find("!=") {
        let (l, r) = t.split_at(idx);
        return l.trim() != r[2..].trim().trim_matches('"');
    }
    if let Some(rest) = t.strip_prefix("not ") {
        return !eval_condition(state, rest);
    }
    true
}
