#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# publish.sh — release rok-auth-macros then rok-auth to crates.io
#
# USAGE
#   ./publish.sh [OPTIONS]
#
# OPTIONS
#   -p, --publish          Actually upload to crates.io (default: dry-run only)
#   -s, --skip-checks      Skip fmt / clippy / test (useful after CI already ran)
#   -y, --yes              Skip the confirmation prompt
#   -h, --help             Show this message
#
# PREREQUISITES
#   cargo login            Must have been run with a valid crates.io API token
#   clean git tree         Required when --publish is used
# ─────────────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")" && pwd)"
MACROS_DIR="$ROOT/rok-auth-macros"

# ── Colours ───────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  C_RED='\033[0;31m' C_GRN='\033[0;32m' C_YLW='\033[1;33m'
  C_CYN='\033[0;36m' C_BLD='\033[1m'    C_RST='\033[0m'
else
  C_RED='' C_GRN='' C_YLW='' C_CYN='' C_BLD='' C_RST=''
fi

step()    { echo -e "\n${C_BLD}${C_CYN}▶ $*${C_RST}"; }
ok()      { echo -e "  ${C_GRN}✓${C_RST}  $*"; }
warn()    { echo -e "  ${C_YLW}!${C_RST}  $*"; }
die()     { echo -e "\n${C_RED}${C_BLD}✗ $*${C_RST}" >&2; exit 1; }
hr()      { echo -e "${C_BLD}$(printf '─%.0s' {1..60})${C_RST}"; }

# ── Argument parsing ──────────────────────────────────────────────────────────
DRY_RUN=true
SKIP_CHECKS=false
AUTO_YES=false

usage() {
  sed -n '4,15p' "$0" | sed 's/^# \?//'
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -p|--publish)     DRY_RUN=false      ;;
    -s|--skip-checks) SKIP_CHECKS=true   ;;
    -y|--yes)         AUTO_YES=true      ;;
    -h|--help)        usage              ;;
    *) die "Unknown argument: $1 — run '$0 --help' for usage" ;;
  esac
  shift
done

# ── Read versions from Cargo.toml ─────────────────────────────────────────────
version_of() {
  grep -m1 '^version' "$1" | sed 's/.*"\(.*\)"/\1/'
}

[[ -f "$MACROS_DIR/Cargo.toml" ]] || die "rok-auth-macros not found at $MACROS_DIR"

MACROS_VER=$(version_of "$MACROS_DIR/Cargo.toml")
MAIN_VER=$(version_of "$ROOT/Cargo.toml")

# ── Header ────────────────────────────────────────────────────────────────────
hr
echo -e " ${C_BLD}rok-auth release script${C_RST}"
hr
echo -e "  rok-auth-macros  ${C_BLD}v${MACROS_VER}${C_RST}"
echo -e "  rok-auth         ${C_BLD}v${MAIN_VER}${C_RST}"
echo -e "  mode             ${C_BLD}$([[ $DRY_RUN == true ]] && echo 'dry-run' || echo 'PUBLISH')${C_RST}"
hr

[[ "$MACROS_VER" == "$MAIN_VER" ]] \
  || warn "Version mismatch — macros=$MACROS_VER, main=$MAIN_VER"

# ── Git sanity ────────────────────────────────────────────────────────────────
step "Checking git state"

if ! git -C "$ROOT" rev-parse --git-dir &>/dev/null; then
  warn "Not a git repository — skipping git checks"
else
  BRANCH=$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)
  ok "Branch: $BRANCH"

  if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
    warn "Not on main/master — are you sure you want to release from '$BRANCH'?"
  fi

  if [[ $DRY_RUN == false ]]; then
    if ! git -C "$ROOT" diff --quiet HEAD 2>/dev/null; then
      die "Working tree has uncommitted changes.\n  Commit or stash everything before publishing."
    fi
    ok "Working tree is clean"

    TAG="v${MAIN_VER}"
    if git -C "$ROOT" rev-parse "$TAG" &>/dev/null; then
      warn "Tag $TAG already exists"
    fi
  fi
fi

# ── Quality checks ────────────────────────────────────────────────────────────
if [[ $SKIP_CHECKS == true ]]; then
  warn "Skipping quality checks (--skip-checks)"
else
  step "Formatting"
  cargo fmt --manifest-path "$ROOT/Cargo.toml" --check \
    || die "Formatting issues found.\n  Run: cargo fmt"
  ok "cargo fmt"

  step "Linting"
  cargo clippy --manifest-path "$ROOT/Cargo.toml" --all-targets -- -D warnings \
    || die "Clippy errors found — fix them before publishing."
  ok "cargo clippy"

  step "Tests"
  cargo test --manifest-path "$ROOT/Cargo.toml" \
    || die "Tests failed — fix them before publishing."
  ok "cargo test"
fi

# ── Dry-run ───────────────────────────────────────────────────────────────────
step "Packaging rok-auth-macros (dry-run)"
cargo publish --dry-run --allow-dirty \
  --manifest-path "$MACROS_DIR/Cargo.toml" \
  || die "rok-auth-macros failed dry-run."
ok "rok-auth-macros packages cleanly"

step "Packaging rok-auth (dry-run)"
# rok-auth dry-run only resolves once rok-auth-macros is live on crates.io.
# Suppress the error on first-time publish — the macros crate isn't indexed yet.
if cargo publish --dry-run --allow-dirty \
     --manifest-path "$ROOT/Cargo.toml" 2>/dev/null; then
  ok "rok-auth packages cleanly"
else
  warn "rok-auth dry-run skipped — rok-auth-macros not yet on crates.io (expected on first release)"
fi

# ── Exit here for dry-run mode ────────────────────────────────────────────────
if [[ $DRY_RUN == true ]]; then
  echo ""
  hr
  ok "Dry-run complete — everything looks good."
  echo -e "  Run ${C_BLD}$0 --publish${C_RST} to upload to crates.io."
  hr
  exit 0
fi

# ── Confirm ───────────────────────────────────────────────────────────────────
echo ""
hr
echo -e " ${C_BLD}${C_YLW}About to publish to crates.io — this cannot be undone.${C_RST}"
echo -e "   1. rok-auth-macros  v${MACROS_VER}"
echo -e "   2. rok-auth         v${MAIN_VER}"
hr
echo ""

if [[ $AUTO_YES == false ]]; then
  read -rp "  Type 'yes' to continue: " confirm
  [[ "$confirm" == "yes" ]] || { echo "  Aborted."; exit 0; }
fi

# ── Publish macros ────────────────────────────────────────────────────────────
step "Publishing rok-auth-macros v${MACROS_VER}"
cargo publish --manifest-path "$MACROS_DIR/Cargo.toml"
ok "rok-auth-macros v${MACROS_VER} uploaded"

# ── Wait for crates.io to index it ───────────────────────────────────────────
step "Waiting for crates.io to index rok-auth-macros"

POLL_MAX=24   # 24 × 5s = 2 minutes
POLL_WAIT=5
indexed=false

for attempt in $(seq 1 $POLL_MAX); do
  printf "\r  Attempt %d/%d …" "$attempt" "$POLL_MAX"

  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    "https://crates.io/api/v1/crates/rok-auth-macros/${MACROS_VER}" \
    -H "User-Agent: rok-auth-publish/1.0")

  if [[ "$http_code" == "200" ]]; then
    echo ""
    ok "rok-auth-macros v${MACROS_VER} is live on crates.io"
    indexed=true
    break
  fi

  sleep $POLL_WAIT
done

if [[ $indexed == false ]]; then
  echo ""
  die "rok-auth-macros was not visible on crates.io after 2 minutes.\n  Publish rok-auth manually once the macros crate is indexed:\n  cargo publish --manifest-path $ROOT/Cargo.toml"
fi

# ── Publish main crate ────────────────────────────────────────────────────────
step "Publishing rok-auth v${MAIN_VER}"
cargo publish --manifest-path "$ROOT/Cargo.toml"
ok "rok-auth v${MAIN_VER} uploaded"

# ── Tag the release ───────────────────────────────────────────────────────────
if git -C "$ROOT" rev-parse --git-dir &>/dev/null; then
  TAG="v${MAIN_VER}"
  if ! git -C "$ROOT" rev-parse "$TAG" &>/dev/null; then
    step "Tagging release $TAG"
    git -C "$ROOT" tag -a "$TAG" -m "Release $TAG"
    ok "Tagged $TAG — push it with: git push origin $TAG"
  fi
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
hr
echo -e " ${C_GRN}${C_BLD}Released successfully!${C_RST}"
echo -e "   https://crates.io/crates/rok-auth-macros/${MACROS_VER}"
echo -e "   https://crates.io/crates/rok-auth/${MAIN_VER}"
echo -e "   https://docs.rs/rok-auth/${MAIN_VER}"
hr
