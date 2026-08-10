use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use tracing::{instrument, trace, warn};

impl Evaluator {
    pub(crate) fn call_builtin(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        let span = tracing::info_span!("call_builtin", name = name, n_args = args.len());
        let _enter = span.enter();
        match name {
            // math
            "abs" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.abs(), u.clone())),
                _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
            },
            "ceil" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.ceil(), u.clone())),
                _ => Err(SassError::Eval("ceil 需要 1 个数字参数".into())),
            },
            "floor" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.floor(), u.clone())),
                _ => Err(SassError::Eval("floor 需要 1 个数字参数".into())),
            },
            "round" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.round(), u.clone())),
                _ => Err(SassError::Eval("round 需要 1 个数字参数".into())),
            },
            "min" => args.iter().try_fold(Value::Number(f64::INFINITY, None), |acc, v| match (acc, v) {
                (Value::Number(a, _), Value::Number(b, u)) => Ok(Value::Number(a.min(*b), u.clone())),
                _ => Err(SassError::Eval("min 需要数字参数".into())),
            }),
            "max" => args.iter().try_fold(Value::Number(f64::NEG_INFINITY, None), |acc, v| match (acc, v) {
                (Value::Number(a, _), Value::Number(b, u)) => Ok(Value::Number(a.max(*b), u.clone())),
                _ => Err(SassError::Eval("max 需要数字参数".into())),
            }),
            "percentage" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval("percentage 需要 1 个数字参数".into())),
            },
            // string
            "str-length" => match args {
                [Value::String(s, _)] => Ok(Value::Number(s.chars().count() as f64, None)),
                _ => Err(SassError::Eval("str-length 需要 1 个字符串参数".into())),
            },
            "to-upper-case" => match args {
                [Value::String(s, q)] => Ok(Value::String(s.to_uppercase(), *q)),
                _ => Err(SassError::Eval("to-upper-case 需要 1 个字符串参数".into())),
            },
            "to-lower-case" => match args {
                [Value::String(s, q)] => Ok(Value::String(s.to_lowercase(), *q)),
                _ => Err(SassError::Eval("to-lower-case 需要 1 个字符串参数".into())),
            },
            "unquote" => match args {
                [Value::String(s, _)] => Ok(Value::String(s.clone(), false)),
                _ => Err(SassError::Eval("unquote 需要 1 个字符串参数".into())),
            },
            "quote" => match args {
                [Value::String(s, _)] => Ok(Value::String(s.clone(), true)),
                _ => Err(SassError::Eval("quote 需要 1 个字符串参数".into())),
            },
            // color
            "rgba" => Self::builtin_rgba(args),
            "rgb" => Self::builtin_rgba(args),
            "darken" => Self::builtin_darken(args),
            "lighten" => Self::builtin_lighten(args),
            "mix" => Self::builtin_mix(args),
            "invert" => match args {
                [Value::Color(c)] => Ok(Value::Color(Color::rgb(255 - c.r, 255 - c.g, 255 - c.b))),
                _ => Err(SassError::Eval("invert 需要 1 个颜色参数".into())),
            },
            "grayscale" => match args {
                [Value::Color(c)] => {
                    let avg = ((c.r as u16 + c.g as u16 + c.b as u16) / 3) as u8;
                    Ok(Value::Color(Color::rgba(avg, avg, avg, c.a)))
                }
                _ => Err(SassError::Eval("grayscale 需要 1 个颜色参数".into())),
            },
            // list
"length" | "list-length" => match args {
[Value::List(es, _, _)] => Ok(Value::Number(es.len() as f64, None)),
[Value::Map(pairs)] => Ok(Value::Number(pairs.len() as f64, None)),
[_] => Ok(Value::Number(1.0, None)),
_ => Err(SassError::Eval("length 需要 1 个参数".into())),
},
"nth" => match args {
[Value::List(es, _, _), Value::Number(n, _)] => {
let len = es.len() as i64;
let idx = *n as i64;
let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
else { return Err(SassError::Eval("nth 索引 0 无效（从 1 开始）".into())); };
es.get(actual).cloned().ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))
}
[Value::Map(pairs), Value::Number(n, _)] => {
let len = pairs.len() as i64;
let idx = *n as i64;
let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
else { return Err(SassError::Eval("nth 索引 0 无效".into())); };
pairs.get(actual).map(|(k, v)| Value::List(vec![k.clone(), v.clone()], Separator::Space, false))
.ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))
}
[other, Value::Number(1.0, _)] => Ok(other.clone()),
[other, Value::Number(-1.0, _)] => Ok(other.clone()),
_ => Err(SassError::Eval("nth 需要 (list, n) 参数".into())),
},
            // map
            "map-get" => match args {
                [Value::Map(pairs), key] => pairs.iter()
                    .find(|(k, _)| Self::values_eq(k, key))
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| SassError::Eval(format!("map-get: 键 {key} 不存在"))),
                _ => Err(SassError::Eval("map-get 需要 (map, key) 参数".into())),
            },
            "map-keys" => match args {
                [Value::Map(pairs)] => Ok(Value::List(pairs.iter().map(|(k, _)| k.clone()).collect(), Separator::Comma, false)),
                _ => Err(SassError::Eval("map-keys 需要 1 个 map 参数".into())),
            },
            "map-values" => match args {
                [Value::Map(pairs)] => Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect(), Separator::Comma, false)),
                _ => Err(SassError::Eval("map-values 需要 1 个 map 参数".into())),
            },
            "map-has-key" => match args {
                [Value::Map(pairs), key] => Ok(Value::Bool(pairs.iter().any(|(k, _)| Self::values_eq(k, key)))),
                _ => Err(SassError::Eval("map-has-key 需要 (map, key) 参数".into())),
            },
            // meta
            "type-of" => match args {
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                _ => Ok(Value::String("unknown".into(), false)),
            },
            "inspect" => match args {
                [v] => Ok(Value::String(Self::inspect_value(v), false)),
                _ => Err(SassError::Eval("inspect 需要 1 个参数".into())),
            },
            "if" => match args {
                [cond, t, f] => Ok(if Self::is_truthy(cond) { t.clone() } else { f.clone() }),
                _ => Err(SassError::Eval("if 需要 3 个参数".into())),
            },
            // string (additional)
            "str-slice" => match args {
                [Value::String(s, q), Value::Number(start, _)] => {
                    let start = *start as isize;
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as isize;
                    let start_idx = if start < 0 { (len + start).max(0) as usize } else { (start - 1).max(0) as usize };
                    let result: String = chars[start_idx.min(len as usize)..].iter().collect();
                    Ok(Value::String(result, *q))
                }
                [Value::String(s, q), Value::Number(start, _), Value::Number(end, _)] => {
                    let start = *start as isize;
                    let end = *end as isize;
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as isize;
                    let start_idx = if start < 0 { (len + start).max(0) as usize } else { (start - 1).max(0) as usize };
                    let end_idx = if end < 0 { (len + end + 1).max(0) as usize } else { end.min(len) as usize };
                    let result: String = chars[start_idx.min(end_idx)..end_idx.min(len as usize)].iter().collect();
                    Ok(Value::String(result, *q))
                }
                _ => Err(SassError::Eval("str-slice 需要 2-3 个参数".into())),
            },
            "str-index" => match args {
                [Value::String(s, _), Value::String(needle, _)] => {
                    match s.find(needle) {
                        Some(pos) => Ok(Value::Number((s[..pos].chars().count() + 1) as f64, None)),
                        None => Ok(Value::Null),
                    }
                }
                _ => Err(SassError::Eval("str-index 需要 2 个字符串参数".into())),
            },
            "str-insert" => match args {
                [Value::String(s, q), Value::String(insert, _), Value::Number(idx, _)] => {
                    let idx = *idx as usize;
                    let chars: Vec<char> = s.chars().collect();
                    let pos = idx.min(chars.len()).min(idx.saturating_sub(1));
                    let mut result: Vec<char> = chars[..pos].to_vec();
                    result.extend(insert.chars());
                    result.extend(chars[pos..].iter());
                    Ok(Value::String(result.into_iter().collect(), *q))
                }
                _ => Err(SassError::Eval("str-insert 需要 3 个参数".into())),
            },
            "str-split" => match args {
                [Value::String(s, _), Value::String(sep, _)] => {
                    let parts: Vec<Value> = if sep.is_empty() {
                        s.chars().map(|c| Value::String(c.to_string(), false)).collect()
                    } else {
                        s.split(sep.as_str()).map(|p| Value::String(p.to_string(), false)).collect()
                    };
                    Ok(Value::List(parts, Separator::Comma, false))
                }
                [Value::String(s, _)] => {
                    let parts: Vec<Value> = s.chars().map(|c| Value::String(c.to_string(), false)).collect();
                    Ok(Value::List(parts, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("str-split 需要 1-2 个参数".into())),
            },
            "unique-id" => Ok(Value::String(format!("id{}", std::time::SystemTime::now().elapsed().unwrap_or_default().as_nanos()), false)),
            // math (additional)
"math.div" | "div" => match args {
[Value::Number(a, u1), Value::Number(b, _)] => {
if *b == 0.0 {
if *a == 0.0 { return Ok(Value::Number(f64::NAN, u1.clone())); }
return Ok(Value::Number(a / b, u1.clone()));
}
Ok(Value::Number(a / b, u1.clone()))
                }
                _ => Err(SassError::Eval("div 需要 2 个数字参数".into())),
            },
            "pow" => match args {
                [Value::Number(a, _), Value::Number(b, _)] => Ok(Value::Number(a.powf(*b), None)),
                _ => Err(SassError::Eval("pow 需要 2 个数字参数".into())),
            },
            "sqrt" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sqrt(), None)),
                _ => Err(SassError::Eval("sqrt 需要 1 个数字参数".into())),
            },
            "sin" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.sin(), None)), _ => Err(SassError::Eval("sin 需要 1 个参数".into())) },
            "cos" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.cos(), None)), _ => Err(SassError::Eval("cos 需要 1 个参数".into())) },
            "tan" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.tan(), None)), _ => Err(SassError::Eval("tan 需要 1 个参数".into())) },
            "random" => match args {
                [] => Ok(Value::Number(Self::simple_random(), None)),
                [Value::Number(n, _)] => Ok(Value::Number((Self::simple_random() * n).floor() + 1.0, None)),
                _ => Err(SassError::Eval("random 需要 0-1 个参数".into())),
            },
            "unit" => match args {
                [Value::Number(_, Some(u))] => Ok(Value::String(u.clone(), false)),
                [Value::Number(_, None)] => Ok(Value::String("".into(), false)),
                _ => Err(SassError::Eval("unit 需要 1 个数字参数".into())),
            },
            "is-unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("is-unitless 需要 1 个数字参数".into())),
            },
            "compatible" => match args {
                [Value::Number(_, None), _] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(u1)), Value::Number(_, Some(u2))] => Ok(Value::Bool(u1 == u2)),
                [Value::Number(_, Some(_)), Value::Number(_, None)] => Ok(Value::Bool(true)),
                _ => Err(SassError::Eval("compatible 需要 2 个数字参数".into())),
            },
            // color (additional)
            "color-channel" => match args {
                [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                    "red" => Ok(Value::Number(c.r as f64, None)),
                    "green" => Ok(Value::Number(c.g as f64, None)),
                    "blue" => Ok(Value::Number(c.b as f64, None)),
                    "alpha" => Ok(Value::Number(c.a as f64, None)),
                    _ => Err(SassError::Eval(format!("未知颜色通道: {ch}"))),
                }
                _ => Err(SassError::Eval("color-channel 需要 (color, channel) 参数".into())),
            },
            "adjust-color" | "change-color" | "scale-color" => {
                args.first().cloned().ok_or_else(|| SassError::Eval("颜色函数需要至少 1 个参数".into()))
            }
            "hwb" => match args {
                [Value::Number(h, _), Value::Number(w, _), Value::Number(b, _)] => {
                    Ok(Value::Color(Self::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, 1.0)))
                }
                [Value::Number(h, _), Value::Number(w, _), Value::Number(b, _), Value::Number(a, _)] => {
                    Ok(Value::Color(Self::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, *a as f32)))
                }
                _ => Err(SassError::Eval("hwb 需要 3-4 个参数".into())),
            }
            "complement" => match args {
                [Value::Color(c)] => Ok(Value::Color(Color::rgb(255 - c.r, 255 - c.g, 255 - c.b))),
                _ => Err(SassError::Eval("complement 需要 1 个颜色参数".into())),
            },
            // map (additional)
            "map-merge" => match args {
                [Value::Map(a), Value::Map(b)] => {
                    let mut merged = a.clone();
                    for (k, v) in b { merged.push((k.clone(), v.clone())); }
                    Ok(Value::Map(merged))
                }
                _ => Err(SassError::Eval("map-merge 需要 2 个 map 参数".into())),
            },
            "map-remove" => match args {
                [Value::Map(pairs), keys @ ..] => {
                    let filtered: Vec<(Value, Value)> = pairs.iter()
                        .filter(|(k, _)| !keys.iter().any(|key| Self::values_eq(k, key)))
                        .cloned()
                        .collect();
                    Ok(Value::Map(filtered))
                }
                [other] => Ok(other.clone()),
                _ => Err(SassError::Eval("map-remove 需要至少 1 个参数".into())),
            },
            "map-deep-remove" => match args {
                [Value::Map(pairs), key @ ..] => {
                    let keys: Vec<&Value> = key.iter().collect();
                    if keys.is_empty() { return Ok(Value::Map(pairs.clone())); }
                    let filtered: Vec<(Value, Value)> = pairs.iter()
                        .filter(|(k, _)| !Self::values_eq(k, keys[0]))
                        .cloned()
                        .collect();
                    Ok(Value::Map(filtered))
                }
                [other, ..] => Ok(other.clone()),
                _ => Err(SassError::Eval("map-deep-remove 需要至少 1 个参数".into())),
            },
            "map-set" => match args {
                [Value::Map(pairs), key, val] => {
                    let mut result = pairs.clone();
                    if let Some(entry) = result.iter_mut().find(|(k, _)| Self::values_eq(k, key)) {
                        entry.1 = val.clone();
                    } else {
                        result.push((key.clone(), val.clone()));
                    }
                    Ok(Value::Map(result))
                }
                [other, _key, _val] => Ok(other.clone()),
                _ => Err(SassError::Eval("map-set 需要 3 个参数".into())),
            },
            // meta (additional)
            "mixin-exists" => Ok(Value::Bool(false)),
            "function-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.get_function(name).is_some())),
                _ => Ok(Value::Bool(false)),
            },
            "global-variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "get-function" => match args {
                [Value::String(fname, _)] => Ok(Value::String(fname.clone(), false)),
                _ => Err(SassError::Eval("get-function 需要 1 个参数".into())),
            },
            "call" => match args {
                [Value::String(fname, _), rest @ ..] => Self::call_builtin(fname, rest, env),
                _ => Err(SassError::Eval("call 需要至少 1 个参数".into())),
            },
            "keywords" => match args {
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords 需要 1 个参数".into())),
            },
            // list (additional)
            "append" => match args {
                [Value::List(items, sep, false), val] => {
                    let mut new_items = items.clone();
                    new_items.push(val.clone());
                    Ok(Value::List(new_items, sep.clone(), false))
                }
                [Value::List(items, sep, false), val, Value::String(s, _)] => {
                    let new_sep = match s.as_str() {
                        "comma" => Separator::Comma,
                        "space" => Separator::Space,
                        "slash" => Separator::Slash,
                        _ => sep.clone(),
                    };
                    let mut new_items = items.clone();
                    new_items.push(val.clone());
                    Ok(Value::List(new_items, new_sep, false))
                }
                [other, val] => {
                    // 处理 append(非列表, 值) 的情况
                    let items = match other {
                        Value::List(items, _, _) => {
                            let mut i = items.clone();
                            i.push(val.clone());
                            i
                        }
                        _ => vec![other.clone(), val.clone()],
                    };
                    Ok(Value::List(items, Separator::Space, false))
                }
                _ => Err(SassError::Eval("append 需要 2-3 个参数".into())),
            },
            "join" => match args {
                [Value::List(a, sa, false), Value::List(b, sb, false)] => {
                    let sep = if a.is_empty() { sb.clone() } else { sa.clone() };
                    let mut items = a.clone();
                    items.extend(b.clone());
                    Ok(Value::List(items, sep, false))
                }
                [Value::List(a, sa, false), Value::List(b, sb, false), Value::String(s, _)] => {
                    let sep = match s.as_str() {
                        "comma" => Separator::Comma,
                        "space" => Separator::Space,
                        "slash" => Separator::Slash,
                        _ => if a.is_empty() { sb.clone() } else { sa.clone() },
                    };
                    let mut items = a.clone();
                    items.extend(b.clone());
                    Ok(Value::List(items, sep, false))
                }
                [a, b] => {
                    // 处理 join((), c) 或 join(c, ()) 的情况
                    let (a_items, a_sep) = match a {
                        Value::List(items, sep, _) => (items.clone(), sep.clone()),
                        _ => (vec![a.clone()], Separator::Undecided),
                    };
                    let (b_items, b_sep) = match b {
                        Value::List(items, sep, _) => (items.clone(), sep.clone()),
                        _ => (vec![b.clone()], Separator::Undecided),
                    };
                    let sep = if a_items.is_empty() { b_sep } else { a_sep };
                    let mut items = a_items;
                    items.extend(b_items);
                    Ok(Value::List(items, sep, false))
                }
                _ => Err(SassError::Eval("join 需要 2-4 个参数".into())),
            },
            "index" => match args {
                [Value::List(items, _, _), needle] => {
                    for (i, item) in items.iter().enumerate() {
                        if Self::values_eq(item, needle) {
                            return Ok(Value::Number((i + 1) as f64, None));
                        }
                    }
                    Ok(Value::Null)
                }
                [other, needle] => {
                    if Self::values_eq(other, needle) { Ok(Value::Number(1.0, None)) }
                    else { Ok(Value::Null) }
                }
                _ => Err(SassError::Eval("index 需要 2 个参数".into())),
            },
            "list-separator" | "separator" => match args {
                [Value::List(_, Separator::Comma, false)] => Ok(Value::String("comma".into(), false)),
                [Value::List(_, Separator::Space, false)] => Ok(Value::String("space".into(), false)),
                [Value::List(_, Separator::Slash, false)] => Ok(Value::String("slash".into(), false)),
                _ => Ok(Value::String("space".into(), false)),
            },
            "set-nth" => match args {
                [Value::List(items, sep, false), Value::Number(n, _), val] => {
                    let idx = *n as usize;
                    let mut new_items = items.clone();
                    if idx >= 1 && idx <= new_items.len() {
                        new_items[idx - 1] = val.clone();
                    }
                    Ok(Value::List(new_items, sep.clone(), false))
                }
                _ => Err(SassError::Eval("set-nth 需要 3 个参数".into())),
            },
            "is-bracketed" => match args {
                [Value::List(_, _, true)] => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(false)),
            },
            "list-slash" => match args {
                [a, b] => Ok(Value::List(vec![a.clone(), b.clone()], Separator::Slash, false)),
                _ => Err(SassError::Eval("list-slash 需要 2 个参数".into())),
            },
            "zip" => match args {
                [Value::List(a, _, _), Value::List(b, _, _)] => {
                    let pairs: Vec<Value> = a.iter().zip(b.iter()).map(|(x, y)| {
                        Value::List(vec![x.clone(), y.clone()], Separator::Space, false)
                    }).collect();
                    Ok(Value::List(pairs, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("zip 需要 2+ 个列表参数".into())),
            },
            // color (additional)
            "hsl" => match args {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _)] => {
                    Ok(Value::Color(Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0)))
                }
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let mut c = Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    c.a = *a as f32;
                    Ok(Value::Color(c))
                }
                _ => Err(SassError::Eval("hsl 需要 3-4 个参数".into())),
            },
            "hsla" => match args {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let mut c = Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    c.a = *a as f32;
                    Ok(Value::Color(c))
                }
                _ => Err(SassError::Eval("hsla 需要 4 个参数".into())),
            },
            "adjust-hue" => match args {
                [Value::Color(c), Value::Number(deg, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    let new_h = (h + *deg).rem_euclid(360.0);
                    Ok(Value::Color(Self::hsl_to_rgb(new_h, s, l)))
                }
                _ => Err(SassError::Eval("adjust-hue 需要 (color, degrees) 参数".into())),
            },
            "saturate" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Color(Self::hsl_to_rgb(h, (s + *amount / 100.0).min(1.0), l)))
                }
                [Value::Number(n, _)] => Ok(Value::String(format!("saturate({})", n), false)),
                _ => Err(SassError::Eval("saturate 需要 (color, amount) 参数".into())),
            },
            "desaturate" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Color(Self::hsl_to_rgb(h, (s - *amount / 100.0).max(0.0), l)))
                }
                _ => Err(SassError::Eval("desaturate 需要 (color, amount) 参数".into())),
            },
            "transparentize" | "fade-out" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    Ok(Value::Color(Color::rgba(c.r, c.g, c.b, (c.a - *amount as f32).max(0.0))))
                }
                _ => Err(SassError::Eval("transparentize 需要 (color, amount) 参数".into())),
            },
            "opacify" | "fade-in" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    Ok(Value::Color(Color::rgba(c.r, c.g, c.b, (c.a + *amount as f32).min(1.0))))
                }
                _ => Err(SassError::Eval("opacify 需要 (color, amount) 参数".into())),
            },
            "alpha" | "opacity" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.a as f64, None)),
                _ => Err(SassError::Eval("alpha 需要 1 个颜色参数".into())),
            },
            "red" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.r as f64, None)),
                _ => Err(SassError::Eval("red 需要 1 个颜色参数".into())),
            },
            "green" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.g as f64, None)),
                _ => Err(SassError::Eval("green 需要 1 个颜色参数".into())),
            },
            "blue" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.b as f64, None)),
                _ => Err(SassError::Eval("blue 需要 1 个颜色参数".into())),
            },
            "hue" => match args {
                [Value::Color(c)] => {
                    let (h, _, _) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(h, Some("deg".into())))
                }
                _ => Err(SassError::Eval("hue 需要 1 个颜色参数".into())),
            },
            "saturation" => match args {
                [Value::Color(c)] => {
                    let (_, s, _) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(s * 100.0, Some("%".into())))
                }
                _ => Err(SassError::Eval("saturation 需要 1 个颜色参数".into())),
            },
            "lightness" => match args {
                [Value::Color(c)] => {
                    let (_, _, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(l * 100.0, Some("%".into())))
                }
                _ => Err(SassError::Eval("lightness 需要 1 个颜色参数".into())),
            },
            // math (additional)
            "clamp" => match args {
                [Value::Number(min, _), Value::Number(val, _), Value::Number(max, _)] => {
                    Ok(Value::Number(val.max(*min).min(*max), None))
                }
                _ => Err(SassError::Eval("clamp 需要 3 个数字参数".into())),
            },
"comparable" => match args {
[Value::Number(_, u1), Value::Number(_, u2)] => {
Ok(Value::Bool(Self::units_compatible(u1.as_deref(), u2.as_deref())))
}
_ => Err(SassError::Eval("comparable 需要 2 个数字参数".into())),
},
            "unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("unitless 需要 1 个数字参数".into())),
            },
            // CSS 原生函数——原样保留
            "calc" | "clamp" | "env" | "var" => {
                let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            },
            // selector functions
            "selector-append" => {
                let parts: Vec<String> = args.iter().map(|a| match a {
                    Value::String(s, _) => s.clone(),
                    _ => a.to_string(),
                }).collect();
                Ok(Value::String(parts.join(""), false))
            }
            "selector-nest" => {
                let parts: Vec<String> = args.iter().map(|a| match a {
                    Value::String(s, _) => s.clone(),
                    _ => a.to_string(),
                }).collect();
                Ok(Value::String(parts.join(" "), false))
            }
            "selector-is-super" => match args {
                [Value::String(a, _), Value::String(b, _)] => {
                    Ok(Value::Bool(b.contains(a.as_str())))
                }
                _ => Ok(Value::Bool(false)),
            }
            "selector-parse" => match args {
                [Value::String(s, _)] => {
                    let parts: Vec<Value> = s.split(',').map(|p| Value::String(p.trim().to_string(), false)).collect();
                    Ok(Value::List(parts, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("selector-parse 需要 1 个参数".into())),
            }
            "selector-simple-selectors" => match args {
                [Value::String(s, _)] => {
                    // 拆分复合选择器为简单选择器
                    let mut result = Vec::new();
                    let mut current = String::new();
                    for c in s.chars() {
                        if c == '.' || c == '#' || c == ':' || c == '[' {
                            if !current.is_empty() { result.push(Value::String(current.clone(), false)); }
                            current = c.to_string();
                        } else {
                            current.push(c);
                        }
                    }
                    if !current.is_empty() { result.push(Value::String(current, false)); }
                    Ok(Value::List(result, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("selector-simple-selectors 需要 1 个参数".into())),
            }
            "selector-unify" => match args {
                [Value::String(a, _), Value::String(b, _)] => {
                    // 简化版：如果一个是另一个的前缀，返回另一个
                    if a.contains(b.as_str()) { Ok(Value::String(a.clone(), false)) }
                    else if b.contains(a.as_str()) { Ok(Value::String(b.clone(), false)) }
                    else { Ok(Value::String(format!("{a}{b}"), false)) }
                }
                _ => Ok(Value::Null),
            }
            "selector-extend" => match args {
                [Value::String(selector, _), Value::String(target, _), Value::String(extender, _)] => {
                    let result = if selector.contains(target.as_str()) {
                        format!("{selector}, {extender}")
                    } else {
                        selector.clone()
                    };
                    Ok(Value::String(result, false))
                }
                _ => Err(SassError::Eval("selector-extend 需要 3 个参数".into())),
            }
            // not a function → 原样输出
            _ => Err(SassError::UndefinedFunction(name.to_string())),
        }
}
}
