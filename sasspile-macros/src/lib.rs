//! sasspile 内建函数注册派生宏。
//!
//! 通过 `#[derive(BuiltinRegistry)]` 将内建函数的名称映射集中到
//! 单一结构体声明，宏自动生成三个 `#[doc(hidden)]` 函数：
//!
//! - `__<struct>_module_builtin_name(name) -> Option<&'static str>`
//! - `__<struct>_is_known(name) -> bool`
//! - `__<struct>_dispatch(name, args, kw, env) -> Option<Result<Value>>`
//!
//! `module_dispatch.rs` 中的统一函数依次调用这些生成函数。

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{DeriveInput, Data, Fields, Ident, Lit, LitStr};

/// 派生宏入口。
#[proc_macro_derive(BuiltinRegistry, attributes(builtin))]
pub fn builtin_registry_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand_builtin_registry(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ─── 属性解析（手工解析 syn 3.0 API）────────────────────────

/// 从 `#[builtin(module = "...", dispatch = "...")]` 解析。
#[allow(dead_code)]
struct BuiltinAttr {
    module: String,
    dispatch: String,
}

fn parse_builtin_attr(input: &DeriveInput) -> syn::Result<BuiltinAttr> {
    let mut module = None;
    let mut dispatch = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("builtin") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("module") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    module = Some(s.value());
                }
                Ok(())
            } else if meta.path.is_ident("dispatch") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    dispatch = Some(s.value());
                }
                Ok(())
            } else {
                Err(meta.error("unknown attribute"))
            }
        })?;
    }

    Ok(BuiltinAttr {
        module: module.ok_or_else(|| {
            syn::Error::new(input.ident.span(), "missing `module` in #[builtin(...)]")
        })?,
        dispatch: dispatch.ok_or_else(|| {
            syn::Error::new(input.ident.span(), "missing `dispatch` in #[builtin(...)]")
        })?,
    })
}

// ─── 字段信息 ────────────────────────────────────────────

struct FieldInfo {
    ident: Ident,
    aliases: Vec<String>,
}

/// 解析字段上的 `#[builtin(alias = "...")]` 属性。
fn parse_field_aliases(field: &syn::Field) -> syn::Result<Vec<String>> {
    let mut aliases = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident("builtin") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("alias") {
                let lit: Lit = meta.value()?.parse()?;
                if let Lit::Str(s) = lit {
                    aliases.push(s.value());
                }
                Ok(())
            } else {
                // module/dispatch 等结构体级属性不会出现在字段上
                Ok(())
            }
        })?;
    }
    Ok(aliases)
}

fn collect_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => return Err(syn::Error::new_spanned(&s.fields, "expected named fields")),
        },
        _ => return Err(syn::Error::new(input.ident.span(), "expected struct")),
    };

    fields
        .iter()
        .map(|f| {
            let ident = f
                .ident
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(f, "missing field name"))?;
            let aliases = parse_field_aliases(f)?;
            Ok(FieldInfo { ident, aliases })
        })
        .collect()
}

// ─── snake_case → kebab-case ────────────────────────────

fn snake_to_kebab(s: &str) -> String {
    s.replace('_', "-")
}

// ─── 代码生成 ────────────────────────────────────────────

fn expand_builtin_registry(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let attr = parse_builtin_attr(input)?;
    let fields = collect_fields(input)?;
    let struct_ident = &input.ident;
    let struct_lower = struct_ident.to_string().to_lowercase();

    // 为每个字段生成完整的名称列表：
    //   - 全局名（kebab-case 字段名）
    //   - 默认模块限定名（module.kebab-case）
    //   - 手动 aliases
    let module = &attr.module;

    // ─── module_builtin_name arms：模块限定名 → 全局名 ───
    // 包含：默认 module.kebab + 手动 aliases
    let mbn_arms: Vec<TokenStream2> = fields
        .iter()
        .flat_map(|f| {
            let kebab = snake_to_kebab(&f.ident.to_string());
            let default_alias = format!("{}.{}", module, kebab);
            // 默认模块限定名 + 手动 aliases
            std::iter::once(default_alias)
                .chain(f.aliases.iter().cloned())
                .map(move |alias| {
                    let alias_lit = LitStr::new(&alias, proc_macro2::Span::call_site());
                    let kebab_lit = LitStr::new(&kebab, proc_macro2::Span::call_site());
                    quote! { #alias_lit => Some(#kebab_lit) }
                })
        })
        .collect();

    // ─── is_known patterns：全局名 + 默认模块限定名 + 手动 aliases ───
    let known_patterns: Vec<LitStr> = fields
        .iter()
        .flat_map(|f| {
            let kebab = snake_to_kebab(&f.ident.to_string());
            let default_alias = format!("{}.{}", module, kebab);
            let mut names = vec![kebab, default_alias];
            names.extend(f.aliases.iter().cloned());
            names
        })
        .map(|name| LitStr::new(&name, proc_macro2::Span::call_site()))
        .collect();

    // ─── dispatch patterns：全局名 + 所有 aliases ───
    let dispatch_patterns: Vec<LitStr> = known_patterns.clone();

    // ─── dispatch 路由代码 ───
    let dispatch = &attr.dispatch;
    let dispatch_ident = format_ident!("{}", dispatch);

    let dispatch_body: TokenStream2 = match dispatch.as_str() {
        "none" => quote! {
            _ => None,
        },
        "map" => quote! {
            #(#dispatch_patterns)|* => {
                let combined = crate::eval::builtin::merge_map_args(pos_args, kw_args, name);
                match crate::eval::Evaluator::call_map_builtin(name, &combined, env) {
                    Ok(Some(v)) => Some(Ok(v)),
                    Ok(None) => Some(Err(crate::error::SassError::UndefinedFunction(name.to_string()))),
                    Err(e) => Some(Err(e)),
                }
            }
        },
        "string" => quote! {
            #(#dispatch_patterns)|* => {
                match crate::eval::Evaluator::call_string_builtin(name, pos_args, kw_args) {
                    Ok(Some(v)) => Some(Ok(v)),
                    Ok(None) => Some(Err(crate::error::SassError::UndefinedFunction(name.to_string()))),
                    Err(e) => Some(Err(e)),
                }
            }
        },
        // math / color / list / selector
        _ => quote! {
            #(#dispatch_patterns)|* => {
                match crate::eval::builtin::#dispatch_ident::call(name, pos_args, kw_args) {
                    Ok(Some(v)) => Some(Ok(v)),
                    Ok(None) => Some(Err(crate::error::SassError::UndefinedFunction(name.to_string()))),
                    Err(e) => Some(Err(e)),
                }
            }
        },
    };

    // ─── 生成三个函数 ───
    let fn_mbn = format_ident!("__{}_module_builtin_name", struct_lower);
    let fn_ik = format_ident!("__{}_is_known", struct_lower);
    let fn_dsp = format_ident!("__{}_dispatch", struct_lower);

    let vis = &input.vis;

    Ok(quote! {
        /// 由 #[derive(BuiltinRegistry)] 生成——模块限定名 → 内建名。
        #[doc(hidden)]
        #vis fn #fn_mbn(name: &str) -> Option<&'static str> {
            match name {
                #(#mbn_arms,)*
                _ => None,
            }
        }

        /// 由 #[derive(BuiltinRegistry)] 生成——检查是否为已知内建函数。
        #[doc(hidden)]
        #vis fn #fn_ik(name: &str) -> bool {
            matches!(name, #(#known_patterns)|*)
        }

        /// 由 #[derive(BuiltinRegistry)] 生成——按模块路由到子模块 call。
        #[doc(hidden)]
        #vis fn #fn_dsp(
            name: &str,
            pos_args: &[crate::parse::ast::Value],
            kw_args: &im::HashMap<String, crate::parse::ast::Value>,
            env: &crate::eval::Env,
        ) -> Option<crate::error::Result<crate::parse::ast::Value>> {
            match name {
                #dispatch_body
                _ => None,
            }
        }
    })
}
