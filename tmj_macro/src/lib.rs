use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Ident, Lit, LitStr, Meta, MetaNameValue};

/// 声明脚本 API 符号：生成小写键常量并登记到 inventory，供 `script_env.txt` 导出。
///
/// 用法：`script_sym!(BG, Type, "背景全局对象");`
///
/// - 第一参数：Rust 常量名（全大写标识符）
/// - 第二参数：`Type` | `Member` | `Function`
/// - 第三参数：中文说明
#[proc_macro]
pub fn script_sym(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ScriptSymInput);
    let ident = &input.name;
    let lower = ident.to_string().to_lowercase();
    let description = input.description.value();
    let category = script_sym_category_tokens(&input.category);

    let expanded = quote! {
        pub const #ident: &str = #lower;
        inventory::submit! {
            crate::utils::ScriptSymEntry {
                const_name: stringify!(#ident),
                value: #lower,
                category: #category,
                description: #description,
                module: module_path!(),
            }
        }
    };

    expanded.into()
}

fn script_sym_category_tokens(category: &Ident) -> proc_macro2::TokenStream {
    match category.to_string().as_str() {
        "Type" => quote! { crate::utils::ScriptSymCategory::Type },
        "Member" => quote! { crate::utils::ScriptSymCategory::Member },
        "Function" => quote! { crate::utils::ScriptSymCategory::Function },
        other => {
            return syn::Error::new_spanned(
                category,
                format!("expected Type, Member, or Function, got `{other}`"),
            )
            .to_compile_error();
        }
    }
}

struct ScriptSymInput {
    name: Ident,
    category: Ident,
    description: LitStr,
}

impl syn::parse::Parse for ScriptSymInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let category = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let description = input.parse()?;
        Ok(Self {
            name,
            category,
            description,
        })
    }
}

/// 声明 VisualElement 的 z_index 常量，并登记到 inventory 供启动时导出参考表。
///
/// 用法：`ve_z_index!(BG, 0, "背景主图层");`
///
/// 展开为同名 `pub const`（`i32`）及一条 `VeZIndexEntry` 记录。
#[proc_macro]
pub fn ve_z_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as VeZIndexInput);
    let name = &input.name;
    let value = &input.value;
    let description = input.description.value();

    let expanded = quote! {
        pub const #name: i32 = #value;
        inventory::submit! {
            crate::pages::behaviour::ve_z_index::VeZIndexEntry {
                name: stringify!(#name),
                value: #value,
                description: #description,
            }
        }
    };

    expanded.into()
}

struct VeZIndexInput {
    name: Ident,
    value: Expr,
    description: LitStr,
}

impl syn::parse::Parse for VeZIndexInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let value = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let description = input.parse()?;
        Ok(Self {
            name,
            value,
            description,
        })
    }
}

/// 自动实现 Typename 特征
/// 用法：
// 使用默认名称（类型名小写）
///```
///#[derive(TypeName)]
///struct MyStruct;
///
///// 自定义名称
///#[derive(TypeName)]
///#[type_name = "custom_name"]
///struct AnotherStruct;
///```
///
#[proc_macro_derive(TypeName, attributes(type_name))]
pub fn derive_type_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 查找 #[type_name = "custom"] 属性
    let type_name_str = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("type_name"))
        .and_then(|attr| {
            if let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta {
                if let Expr::Lit(expr_lit) = value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| name.to_string().to_lowercase()); // 默认：类型名小写

    let expanded = quote! {
        impl #impl_generics TypeName for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #type_name_str;
        }
    };

    TokenStream::from(expanded)
}
