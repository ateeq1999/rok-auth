use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

const LAYOUT: &str = include_str!("../templates/layout.html");
const HOME_HERO: &str = include_str!("../templates/home.html");

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    docs: Arc<RwLock<HashMap<String, String>>>,
    nav: Arc<Vec<NavItem>>,
}

struct NavItem {
    title: &'static str,
    path: &'static str,
    category: &'static str,
    icon: &'static str,
}

// ── Template helpers ─────────────────────────────────────────────────────────

fn render(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |acc, (k, v)| {
        acc.replace(&format!("{{{{{}}}}}", k), v)
    })
}

fn build_nav(items: &[NavItem], active: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut groups: HashMap<&str, Vec<&NavItem>> = HashMap::new();

    for item in items {
        if !seen.contains(&item.category) {
            seen.push(item.category);
        }
        groups.entry(item.category).or_default().push(item);
    }

    let mut html = String::from(
        r#"<nav class="sidebar">
  <div class="nav-logo">
    <div class="nav-logo-icon">&#x1F510;</div>
    <div class="nav-logo-name">rok-auth</div>
    <span class="nav-logo-badge">v0.1</span>
  </div>
  <div class="nav-body">"#,
    );

    for cat in &seen {
        html.push_str(&format!(
            r#"<div class="nav-section"><div class="nav-section-label">{}</div>"#,
            cat
        ));
        for item in &groups[cat] {
            let cls = if item.path == active { " active" } else { "" };
            html.push_str(&format!(
                r#"<a href="/docs/{}" class="nav-link{}"><span class="icon">{}</span>{}</a>"#,
                item.path, cls, item.icon, item.title
            ));
        }
        html.push_str("</div>");
    }

    html.push_str(
        r#"  </div>
  <div class="nav-footer">
    <a href="/">Home</a>
    <a href="https://github.com/rok/rok-auth" target="_blank">GitHub &#x2197;</a>
  </div>
</nav>"#,
    );
    html
}

// ── Markdown parser ──────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Format inline markdown: **bold**, *italic*, `code`, [link](url).
fn fmt_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            // Bold: **...**
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut inner = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'*') {
                        chars.next();
                        closed = true;
                        break;
                    }
                    inner.push(c);
                }
                if closed {
                    out.push_str("<strong>");
                    out.push_str(&escape_html(&inner));
                    out.push_str("</strong>");
                } else {
                    out.push_str("**");
                    out.push_str(&escape_html(&inner));
                }
            }
            // Italic: *...*
            '*' => {
                let mut inner = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '*' { closed = true; break; }
                    inner.push(c);
                }
                if closed {
                    out.push_str("<em>");
                    out.push_str(&escape_html(&inner));
                    out.push_str("</em>");
                } else {
                    out.push('*');
                    out.push_str(&escape_html(&inner));
                }
            }
            // Inline code: `...`
            '`' => {
                let mut inner = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '`' { closed = true; break; }
                    inner.push(c);
                }
                if closed {
                    out.push_str("<code>");
                    out.push_str(&escape_html(&inner));
                    out.push_str("</code>");
                } else {
                    out.push('`');
                    out.push_str(&escape_html(&inner));
                }
            }
            // Link: [text](url)
            '[' => {
                let text: String = chars.by_ref().take_while(|&c| c != ']').collect();
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let url: String = chars.by_ref().take_while(|&c| c != ')').collect();
                    out.push_str(&format!(
                        r#"<a href="{}">{}</a>"#,
                        escape_html(&url),
                        escape_html(&text)
                    ));
                } else {
                    out.push('[');
                    out.push_str(&escape_html(&text));
                }
            }
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

struct MdState {
    out: String,
    in_code: bool,
    in_list: bool,
    in_olist: bool,
    in_table: bool,
    table_body: bool,
}

impl MdState {
    fn new() -> Self {
        Self {
            out: String::new(),
            in_code: false,
            in_list: false,
            in_olist: false,
            in_table: false,
            table_body: false,
        }
    }

    fn close_table(&mut self) {
        if self.in_table {
            if self.table_body { self.out.push_str("</tbody>"); }
            self.out.push_str("</table>\n");
            self.in_table = false;
            self.table_body = false;
        }
    }

    fn close_lists(&mut self) {
        if self.in_list  { self.out.push_str("</ul>\n");  self.in_list  = false; }
        if self.in_olist { self.out.push_str("</ol>\n"); self.in_olist = false; }
    }

    fn close_all(&mut self) {
        self.close_lists();
        self.close_table();
    }
}

fn markdown_to_html(md: &str) -> String {
    let mut s = MdState::new();

    for line in md.lines() {
        // ── Code fence ──────────────────────────────────────────────────
        if line.starts_with("```") {
            if s.in_code {
                s.out.push_str("</code></pre>\n");
                s.in_code = false;
            } else {
                s.close_all();
                s.out.push_str("<pre><code>");
                s.in_code = true;
            }
            continue;
        }
        if s.in_code {
            s.out.push_str(&escape_html(line));
            s.out.push('\n');
            continue;
        }

        // ── Blank line ───────────────────────────────────────────────────
        if line.trim().is_empty() {
            s.close_all();
            continue;
        }

        // ── Table separator: |---|---| ────────────────────────────────────
        if line.starts_with('|')
            && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
        {
            if s.in_table && !s.table_body {
                s.out.push_str("</thead><tbody>\n");
                s.table_body = true;
            }
            continue;
        }

        // ── Table row ────────────────────────────────────────────────────
        if line.starts_with("| ") || (s.in_table && line.starts_with('|')) {
            if !s.in_table {
                s.close_all();
                s.out.push_str("<table><thead>\n");
                s.in_table = true;
            }
            let tag = if s.table_body { "td" } else { "th" };
            s.out.push_str("<tr>");
            for cell in line.trim_matches('|').split('|') {
                s.out.push_str(&format!(
                    "<{0}>{1}</{0}>",
                    tag,
                    fmt_inline(cell.trim())
                ));
            }
            s.out.push_str("</tr>\n");
            continue;
        }

        // Non-table line: close table if open
        s.close_table();

        // ── Headings ─────────────────────────────────────────────────────
        if let Some(r) = line.strip_prefix("### ") {
            s.close_lists();
            s.out.push_str(&format!("<h3>{}</h3>\n", fmt_inline(r)));
            continue;
        }
        if let Some(r) = line.strip_prefix("## ") {
            s.close_lists();
            s.out.push_str(&format!("<h2>{}</h2>\n", fmt_inline(r)));
            continue;
        }
        if let Some(r) = line.strip_prefix("# ") {
            s.close_lists();
            s.out.push_str(&format!("<h1>{}</h1>\n", fmt_inline(r)));
            continue;
        }

        // ── Blockquote ───────────────────────────────────────────────────
        if let Some(r) = line.strip_prefix("> ") {
            s.close_all();
            s.out.push_str(&format!("<blockquote>{}</blockquote>\n", fmt_inline(r)));
            continue;
        }

        // ── Horizontal rule ──────────────────────────────────────────────
        if matches!(line.trim(), "---" | "***" | "___") {
            s.close_all();
            s.out.push_str("<hr>\n");
            continue;
        }

        // ── Checkbox list: - [x] / - [ ] ─────────────────────────────────
        if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            if !s.in_list {
                if s.in_olist { s.out.push_str("</ol>\n"); s.in_olist = false; }
                s.out.push_str("<ul class=\"checklist\">\n");
                s.in_list = true;
            }
            s.out.push_str(&format!(
                "<li class=\"check-item\"><span class=\"check\">&#x2713;</span> {}</li>\n",
                fmt_inline(&line[6..])
            ));
            continue;
        }
        if line.starts_with("- [ ] ") {
            if !s.in_list {
                if s.in_olist { s.out.push_str("</ol>\n"); s.in_olist = false; }
                s.out.push_str("<ul class=\"checklist\">\n");
                s.in_list = true;
            }
            s.out.push_str(&format!(
                "<li class=\"check-item\"><span class=\"uncheck\">&#x25CB;</span> {}</li>\n",
                fmt_inline(&line[6..])
            ));
            continue;
        }

        // ── Unordered list ───────────────────────────────────────────────
        if let Some(r) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            if !s.in_list {
                if s.in_olist { s.out.push_str("</ol>\n"); s.in_olist = false; }
                s.out.push_str("<ul>\n");
                s.in_list = true;
            }
            s.out.push_str(&format!("<li>{}</li>\n", fmt_inline(r)));
            continue;
        }

        // ── Ordered list: "1. ", "2. " etc. ─────────────────────────────
        if let Some(dot) = line.find(". ") {
            if dot > 0
                && dot <= 3
                && line[..dot].chars().all(|c| c.is_ascii_digit())
            {
                if !s.in_olist {
                    if s.in_list { s.out.push_str("</ul>\n"); s.in_list = false; }
                    s.out.push_str("<ol>\n");
                    s.in_olist = true;
                }
                s.out.push_str(&format!("<li>{}</li>\n", fmt_inline(&line[dot + 2..])));
                continue;
            }
        }

        // ── Paragraph ────────────────────────────────────────────────────
        s.close_lists();
        s.out.push_str(&format!("<p>{}</p>\n", fmt_inline(line)));
    }

    s.close_all();
    s.out
}

// ── Docs loader ──────────────────────────────────────────────────────────────

fn load_docs(path: &PathBuf) -> HashMap<String, String> {
    let mut docs = HashMap::new();
    let Ok(entries) = std::fs::read_dir(path) else { return docs };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().map_or(false, |e| e == "md") {
            if let Some(name) = p.file_stem().and_then(|n| n.to_str()) {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    docs.insert(name.to_string(), content);
                }
            }
        }
    }
    docs
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn home(State(state): State<AppState>) -> Html<String> {
    let nav = build_nav(&state.nav, "");
    let content = HOME_HERO.to_string();
    Html(render(
        LAYOUT,
        &[
            ("TITLE", "rok-auth — Rust Authentication Library"),
            ("NAV", &nav),
            ("BREADCRUMB", ""),
            ("CONTENT", &content),
        ],
    ))
}

async fn docs_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Response {
    let page = path.trim_end_matches(".md");
    let docs = state.docs.read().await;
    let doc = docs
        .get(page)
        .or_else(|| docs.iter().find(|(k, _)| k.contains(page)).map(|(_, v)| v))
        .cloned()
        .unwrap_or_default();
    drop(docs);

    if doc.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Html(format!(
                "<h1>404</h1><p>Document <code>{}</code> not found.</p>",
                escape_html(page)
            )),
        )
            .into_response();
    }

    let nav = build_nav(&state.nav, page);
    let content = markdown_to_html(&doc);
    let title = format!("{} — rok-auth", page.replace('-', " "));
    let breadcrumb = format!(
        r#"<div class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span>{}</div>"#,
        escape_html(page)
    );

    Html(render(
        LAYOUT,
        &[
            ("TITLE", &title),
            ("NAV", &nav),
            ("BREADCRUMB", &breadcrumb),
            ("CONTENT", &content),
        ],
    ))
    .into_response()
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let docs = load_docs(&PathBuf::from("docs"));

    let nav = Arc::new(vec![
        NavItem { title: "Overview",           path: "progress",                       category: "Getting Started", icon: "&#x1F4CA;" },
        NavItem { title: "Phase 1: Core Auth", path: "phase-01-core-authentication",   category: "Phases",          icon: "&#x1F510;" },
        NavItem { title: "Phase 2: User Auth", path: "phase-02-user-authentication",   category: "Phases",          icon: "&#x1F464;" },
        NavItem { title: "Phase 3: 2FA",       path: "phase-03-two-factor-auth",       category: "Phases",          icon: "&#x1F4F1;" },
        NavItem { title: "Phase 4: OAuth",     path: "phase-04-oauth-integration",     category: "Phases",          icon: "&#x1F310;" },
        NavItem { title: "Phase 5: Email",     path: "phase-05-email-verification",    category: "Phases",          icon: "&#x1F4E7;" },
        NavItem { title: "Phase 6: RBAC",      path: "phase-06-rbac-advanced",         category: "Phases",          icon: "&#x1F465;" },
        NavItem { title: "Phase 7: API Polish",path: "phase-07-api-polish",            category: "Phases",          icon: "&#x2728;" },
        NavItem { title: "Phase 8: Security",  path: "phase-08-rate-limiting-security",category: "Phases",          icon: "&#x1F6E1;" },
        NavItem { title: "Phase 9: CLI Spec",  path: "phase-09-cli-commands",          category: "Phases",          icon: "&#x1F4BB;" },
        NavItem { title: "CLI Commands",       path: "commands",                       category: "Reference",       icon: "&#x1F4DC;" },
        NavItem { title: "Dev Notes",          path: "dev",                            category: "Reference",       icon: "&#x1F527;" },
    ]);

    let state = AppState {
        docs: Arc::new(RwLock::new(docs)),
        nav,
    };

    let app = Router::new()
        .route("/", get(home))
        .route("/docs/{path}", get(docs_handler))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("Documentation server running at http://localhost:4000");
    axum::serve(listener, app).await.unwrap();
}
