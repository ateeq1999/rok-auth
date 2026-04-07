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

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemFn, LitStr, Pat, Token,
};

// ── argument parsers ──────────────────────────────────────────────────────────

struct RoleArg {
    role: LitStr,
}

impl Parse for RoleArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RoleArg { role: input.parse()? })
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

/// Find the identifier of the first `claims: Claims` parameter in the function
/// signature so we know which variable to check.
fn find_claims_ident(func: &ItemFn) -> Option<syn::Ident> {
    for arg in &func.sig.inputs {
        let FnArg::Typed(pat_type) = arg else { continue };
        let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else { continue };
        // Accept any parameter whose type path ends in "Claims"
        if let syn::Type::Path(tp) = pat_type.ty.as_ref() {
            if tp.path.segments.last().map_or(false, |s| s.ident == "Claims") {
                return Some(pat_ident.ident.clone());
            }
        }
    }
    None
}

/// Build the forbidden-response expression used by both macros.
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

// ── shared rewrite helper ─────────────────────────────────────────────────────

/// Rewrites a function so that:
/// 1. Its return type becomes `::axum::response::Response` (concrete type,
///    avoids "two different `impl IntoResponse` types" errors).
/// 2. The guard `if` statement is prepended to the body.
/// 3. The original body is wrapped so its final value is converted via
///    `IntoResponse::into_response()`, matching the new return type.
fn rewrite_fn_with_guard(func: &mut ItemFn, guard: proc_macro2::TokenStream) {
    // Rewrite return type to the concrete axum Response.
    func.sig.output = syn::parse2(quote! { -> ::axum::response::Response })
        .expect("return type parse failed");

    // Collect original body statements and wrap their result with
    // into_response() so the return type matches the new signature.
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

// ── #[require_role("role")] ───────────────────────────────────────────────────

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

// ── #[require_any_role("a", "b")] ────────────────────────────────────────────

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

    let roles_display = roles.iter().map(|r| r.value()).collect::<Vec<_>>().join(" | ");
    let forbidden = forbidden_response(quote! { #roles_display });

    let guard = quote! {
        if !#claims_ident.has_any_role(&[#(#roles),*]) {
            #forbidden
        }
    };

    rewrite_fn_with_guard(&mut func, guard);
    quote! { #func }.into()
}
