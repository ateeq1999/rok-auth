//! Procedural macros for rok-auth.
//!
//! Provides attribute macros that inject role-checks into Axum handler
//! functions at compile time, avoiding the need for boilerplate guards in
//! every handler body.
//!
//! # Usage
//!
//! Add to `Cargo.toml`:
//! ```toml
//! rok-auth-macros = { path = "../rok-auth-macros" }
//! ```
//!
//! Then annotate your handlers:
//!
//! ```rust,ignore
//! use rok_auth_macros::require_role;
//! use rok_auth::Claims;
//! use axum::response::IntoResponse;
//!
//! #[require_role("admin")]
//! async fn admin_dashboard(claims: Claims) -> impl IntoResponse {
//!     format!("Welcome, admin {}", claims.sub)
//! }
//! ```
//!
//! The macro rewrites the function so the **first** `Claims` parameter is used
//! for the role check.  If the role is absent the handler returns a 403 JSON
//! response immediately.
//!
//! # Derive Macros
//!
//! Use `#[derive(UserProvider)]` on your user struct:
//!
//! ```rust,ignore
//! #[derive(UserProvider)]
//! struct User {
//!     id: String,
//!     email: String,
//!     password_hash: String,
//!     roles: Vec<String>,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Data, DeriveInput, Field, FnArg, Ident, ItemFn, LitStr, Pat,
    Token,
};

// ── argument parsers ──────────────────────────────────────────────────────────

struct RoleArg {
    role: LitStr,
}

impl Parse for RoleArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RoleArg {
            role: input.parse()?,
        })
    }
}

struct AnyRoleArgs {
    roles: Vec<LitStr>,
}

impl Parse for AnyRoleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut roles = Vec::new();
        roles.push(input.parse::<LitStr>()?);
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            roles.push(input.parse::<LitStr>()?);
        }
        Ok(AnyRoleArgs { roles })
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn find_claims_ident(func: &ItemFn) -> Option<syn::Ident> {
    for arg in &func.sig.inputs {
        let FnArg::Typed(pat_type) = arg else {
            continue;
        };
        let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            continue;
        };
        if let syn::Type::Path(tp) = pat_type.ty.as_ref() {
            if tp
                .path
                .segments
                .last()
                .map_or(false, |s| s.ident == "Claims")
            {
                return Some(pat_ident.ident.clone());
            }
        }
    }
    None
}

fn forbidden_response(role_display: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        {
            use ::axum::response::IntoResponse;
            return (
                ::axum::http::StatusCode::FORBIDDEN,
                ::axum::Json(::serde_json::json!({
                    "error": "forbidden",
                    "message": format!("role required: {}", #role_display)
                }))
            ).into_response();
        }
    }
}

fn rewrite_fn_with_guard(func: &mut ItemFn, guard: proc_macro2::TokenStream) {
    func.sig.output =
        syn::parse2(quote! { -> ::axum::response::Response }).expect("return type parse failed");

    let original_stmts = func.block.stmts.drain(..).collect::<Vec<_>>();

    *func.block = syn::parse2(quote! {
        {
            #guard
            let __body_result = { #(#original_stmts)* };
            ::axum::response::IntoResponse::into_response(__body_result)
        }
    })
    .expect("block rewrite failed");
}

/// Attribute macro that injects a role-check before the handler body.
///
/// Finds the first `Claims`-typed parameter and calls `has_role(role)` on it.
/// If the check fails the handler returns `403 Forbidden` with a JSON body.
///
/// # Example
///
/// ```rust,ignore
/// #[require_role("admin")]
/// async fn admin_only(claims: Claims) -> impl IntoResponse {
///     "you are an admin"
/// }
/// ```
#[proc_macro_attribute]
pub fn require_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let RoleArg { role } = parse_macro_input!(attr as RoleArg);
    let mut func = parse_macro_input!(item as ItemFn);

    let claims_ident = match find_claims_ident(&func) {
        Some(id) => id,
        None => {
            return syn::Error::new(
                Span::call_site(),
                "#[require_role] requires a `claims: Claims` parameter in the function signature",
            )
            .to_compile_error()
            .into();
        }
    };

    let role_val = role.value();
    let forbidden = forbidden_response(quote! { #role_val });

    let guard = quote! {
        if !#claims_ident.has_role(#role) {
            #forbidden
        }
    };

    rewrite_fn_with_guard(&mut func, guard);
    quote! { #func }.into()
}

/// Attribute macro that injects an any-of-roles check before the handler body.
///
/// Returns `403 Forbidden` if the `Claims` parameter does not contain at least
/// one of the listed roles.
///
/// # Example
///
/// ```rust,ignore
/// #[require_any_role("admin", "moderator")]
/// async fn mod_panel(claims: Claims) -> impl IntoResponse {
///     "you are a mod or admin"
/// }
/// ```
#[proc_macro_attribute]
pub fn require_any_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AnyRoleArgs { roles } = parse_macro_input!(attr as AnyRoleArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    let claims_ident = match find_claims_ident(&func) {
        Some(id) => id,
        None => {
            return syn::Error::new(
                Span::call_site(),
                "#[require_any_role] requires a `claims: Claims` parameter in the function signature",
            )
            .to_compile_error()
            .into();
        }
    };

    let roles_display = roles
        .iter()
        .map(|r| r.value())
        .collect::<Vec<_>>()
        .join(" | ");
    let forbidden = forbidden_response(quote! { #roles_display });

    let guard = quote! {
        if !#claims_ident.has_any_role(&[#(#roles),*]) {
            #forbidden
        }
    };

    rewrite_fn_with_guard(&mut func, guard);
    quote! { #func }.into()
}

// ── Derive macros ───────────────────────────────────────────────────────────────

#[proc_macro_derive(UserProvider, attributes(user_provider))]
pub fn derive_user_provider(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let id_type = get_field_type(&input.data, "id").unwrap_or_else(|| {
        syn::Error::new(Span::call_site(), "Field 'id' not found").to_compile_error()
    });

    let expanded = quote! {
        impl #impl_generics rok_auth::providers::UserProvider for #name #ty_generics #where_clause {
            type Id = #id_type;

            fn user_id(&self) -> Self::Id {
                self.id.clone()
            }

            fn password_hash(&self) -> &str {
                &self.password_hash
            }

            fn roles(&self) -> Vec<String> {
                self.roles.clone()
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_field_type(data: &Data, field_name: &str) -> Option<proc_macro2::TokenStream> {
    let fields = match data {
        Data::Struct(s) => &s.fields,
        _ => return None,
    };

    for field in fields {
        if let Some(ident) = &field.ident {
            if ident == field_name {
                return Some(quote::quote!(#field.ty));
            }
        }
    }
    None
}
