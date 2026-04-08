//! Procedural macros for rok-auth.
//!
//! You do not need to add this crate directly — import macros through `rok_auth::macros::*`
//! or via the re-exports at the `rok_auth` crate root.
//!
//! # Attribute macros
//!
//! | Macro | Description |
//! |-------|-------------|
//! | `#[require_role("admin")]` | 403 if the `Claims` param lacks the role |
//! | `#[require_any_role("a","b")]` | 403 if the `Claims` param has none of the roles |
//! | `#[require_all_roles("a","b")]` | 403 if the `Claims` param lacks any listed role |
//! | `#[require_fresh(secs = 300)]` | 403 if the token was issued more than N seconds ago |
//!
//! # Derive macros
//!
//! | Macro | Description |
//! |-------|-------------|
//! | `#[derive(UserProvider)]` | Implement `UserProvider` for a struct with `id`, `password_hash`, `roles` fields |

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, FnArg, ItemFn, LitInt, LitStr, Pat, Token,
};

// ── Argument parsers ──────────────────────────────────────────────────────────

struct RoleArg {
    role: LitStr,
}
impl Parse for RoleArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RoleArg { role: input.parse()? })
    }
}

struct ManyRoleArgs {
    roles: Vec<LitStr>,
}
impl Parse for ManyRoleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut roles = Vec::new();
        roles.push(input.parse::<LitStr>()?);
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() { break; }
            roles.push(input.parse::<LitStr>()?);
        }
        Ok(ManyRoleArgs { roles })
    }
}

// `#[require_fresh]` or `#[require_fresh(secs = 300)]`
struct FreshArg {
    secs: u64,
}
impl Parse for FreshArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(FreshArg { secs: 300 });
        }
        // parse `secs = <number>`
        let ident: syn::Ident = input.parse()?;
        if ident != "secs" {
            return Err(syn::Error::new(ident.span(), "expected `secs = <number>`"));
        }
        input.parse::<Token![=]>()?;
        let n: LitInt = input.parse()?;
        Ok(FreshArg { secs: n.base10_parse()? })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Find the first `claims: Claims` parameter in a function signature.
fn find_claims_ident(func: &ItemFn) -> Option<syn::Ident> {
    for arg in &func.sig.inputs {
        let FnArg::Typed(pat_type) = arg else { continue };
        let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else { continue };
        if let syn::Type::Path(tp) = pat_type.ty.as_ref() {
            if tp.path.segments.last().map_or(false, |s| s.ident == "Claims") {
                return Some(pat_ident.ident.clone());
            }
        }
    }
    None
}

/// Emit a 403 JSON response.
fn forbidden_json(message: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        {
            use ::axum::response::IntoResponse;
            return (
                ::axum::http::StatusCode::FORBIDDEN,
                ::axum::Json(::serde_json::json!({
                    "error": "forbidden",
                    "message": #message
                }))
            ).into_response();
        }
    }
}

/// Prepend `guard` to the function body and force the return type to `Response`.
fn prepend_guard(func: &mut ItemFn, guard: proc_macro2::TokenStream) {
    func.sig.output =
        syn::parse2(quote! { -> ::axum::response::Response }).expect("return type");
    let stmts = func.block.stmts.drain(..).collect::<Vec<_>>();
    *func.block = syn::parse2(quote! {
        {
            #guard
            let __result = { #(#stmts)* };
            ::axum::response::IntoResponse::into_response(__result)
        }
    })
    .expect("block rewrite");
}

/// Emit a compile error if there is no `Claims` parameter.
fn require_claims(func: &ItemFn, macro_name: &str) -> Result<syn::Ident, TokenStream> {
    find_claims_ident(func).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            format!(
                "#[{macro_name}] requires a `claims: Claims` parameter in the function signature"
            ),
        )
        .to_compile_error()
        .into()
    })
}

// ── #[require_role("role")] ───────────────────────────────────────────────────

/// Guard an Axum handler so it returns `403 Forbidden` when the `Claims`
/// parameter does not contain the specified role.
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

    let claims = match require_claims(&func, "require_role") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let role_str = role.value();
    let forbidden = forbidden_json(quote! { format!("required role: {}", #role_str) });
    let guard = quote! { if !#claims.has_role(#role) { #forbidden } };

    prepend_guard(&mut func, guard);
    quote! { #func }.into()
}

// ── #[require_any_role("a", "b", …)] ─────────────────────────────────────────

/// Guard an Axum handler so it returns `403 Forbidden` when the `Claims`
/// parameter contains **none** of the listed roles.
///
/// ```rust,ignore
/// #[require_any_role("admin", "moderator")]
/// async fn mod_panel(claims: Claims) -> impl IntoResponse {
///     "admin or mod"
/// }
/// ```
#[proc_macro_attribute]
pub fn require_any_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ManyRoleArgs { roles } = parse_macro_input!(attr as ManyRoleArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    let claims = match require_claims(&func, "require_any_role") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let display = roles.iter().map(|r| r.value()).collect::<Vec<_>>().join(" | ");
    let forbidden = forbidden_json(quote! { format!("one of these roles required: {}", #display) });
    let guard = quote! { if !#claims.has_any_role(&[#(#roles),*]) { #forbidden } };

    prepend_guard(&mut func, guard);
    quote! { #func }.into()
}

// ── #[require_all_roles("a", "b", …)] ────────────────────────────────────────

/// Guard an Axum handler so it returns `403 Forbidden` when the `Claims`
/// parameter is missing **any** of the listed roles.
///
/// ```rust,ignore
/// #[require_all_roles("editor", "verified")]
/// async fn publish(claims: Claims) -> impl IntoResponse {
///     "you have all required roles"
/// }
/// ```
#[proc_macro_attribute]
pub fn require_all_roles(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ManyRoleArgs { roles } = parse_macro_input!(attr as ManyRoleArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    let claims = match require_claims(&func, "require_all_roles") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let display = roles.iter().map(|r| r.value()).collect::<Vec<_>>().join(" + ");
    let forbidden = forbidden_json(quote! { format!("all of these roles required: {}", #display) });
    let guard = quote! { if !#claims.has_all_roles(&[#(#roles),*]) { #forbidden } };

    prepend_guard(&mut func, guard);
    quote! { #func }.into()
}

// ── #[require_fresh] / #[require_fresh(secs = 300)] ──────────────────────────

/// Guard an Axum handler so it returns `403 Forbidden` when the `Claims`
/// token was issued more than `secs` seconds ago (default: 300 = 5 minutes).
///
/// Use this for sensitive operations (password change, payment, delete account)
/// that should require a recently-authenticated session.
///
/// ```rust,ignore
/// #[require_fresh(secs = 120)]
/// async fn change_password(claims: Claims) -> impl IntoResponse {
///     "token is fresh — allow the sensitive operation"
/// }
/// ```
#[proc_macro_attribute]
pub fn require_fresh(attr: TokenStream, item: TokenStream) -> TokenStream {
    let FreshArg { secs } = parse_macro_input!(attr as FreshArg);
    let mut func = parse_macro_input!(item as ItemFn);

    let claims = match require_claims(&func, "require_fresh") {
        Ok(id) => id,
        Err(e) => return e,
    };

    let forbidden = forbidden_json(
        quote! { format!("token too old — re-authenticate within {}s for this action", #secs) },
    );
    let guard = quote! {
        {
            let __now = ::std::time::SystemTime::now()
                .duration_since(::std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            if __now - #claims.iat > #secs as i64 {
                #forbidden
            }
        }
    };

    prepend_guard(&mut func, guard);
    quote! { #func }.into()
}

// ── #[derive(UserProvider)] ───────────────────────────────────────────────────

/// Derive [`rok_auth::UserProvider`] for a struct.
///
/// The struct must have fields named `id: String`, `password_hash: String`,
/// and `roles: Vec<String>`.
///
/// ```rust,ignore
/// use rok_auth_macros::UserProvider;
///
/// #[derive(UserProvider)]
/// struct User {
///     id: String,
///     email: String,          // extra fields are fine
///     password_hash: String,
///     roles: Vec<String>,
/// }
/// ```
#[proc_macro_derive(UserProvider)]
pub fn derive_user_provider(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Validate required fields exist
    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return syn::Error::new(Span::call_site(), "#[derive(UserProvider)] only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let required = ["id", "password_hash", "roles"];
    for req in &required {
        if !fields.iter().any(|f| f.ident.as_ref().map_or(false, |i| i == req)) {
            return syn::Error::new(
                Span::call_site(),
                format!("#[derive(UserProvider)] requires a field named `{req}`"),
            )
            .to_compile_error()
            .into();
        }
    }

    quote! {
        impl #impl_generics ::rok_auth::UserProvider for #name #ty_generics #where_clause {
            fn user_id(&self) -> String {
                self.id.clone()
            }

            fn password_hash(&self) -> &str {
                &self.password_hash
            }

            fn roles(&self) -> Vec<String> {
                self.roles.clone()
            }
        }
    }
    .into()
}
