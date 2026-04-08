#!/usr/bin/env bash
# publish.sh — publish rok-auth-macros then rok-auth to crates.io
#
# Usage:
#   ./publish.sh           # dry-run only (safe)
#   ./publish.sh --publish # actually publish both crates
#
# Requirements:
#   - cargo login must have been run with a valid crates.io token
#   - working tree must be clean (committed)
#   - both crates must be at the correct version in their Cargo.toml

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}[info]${RESET}  $*"; }
success() { echo -e "${GREEN}${BOLD}[ok]${RESET}    $*"; }
warn()    { echo -e "${YELLOW}${BOLD}[warn]${RESET}  $*"; }
die()     { echo -e "${RED}${BOLD}[error]${RESET} $*" >&2; exit 1; }

# ── Parse arguments ───────────────────────────────────────────────────────────
DRY_RUN=true
for arg in "$@"; do
  case "$arg" in
    --publish) DRY_RUN=false ;;
    --help|-h)
      echo "Usage: $0 [--publish]"
      echo "  (no flags)   Dry-run only — nothing is uploaded"
      echo "  --publish    Actually publish both crates to crates.io"
      exit 0 ;;
    *) die "Unknown argument: $arg" ;;
  esac
done

# ── Preflight ─────────────────────────────────────────────────────────────────
MACROS_DIR="$(dirname "$0")/rok-auth-macros"
ROOT_DIR="$(dirname "$0")"

[[ -f "$MACROS_DIR/Cargo.toml" ]] || die "rok-auth-macros not found at $MACROS_DIR"

MACROS_VERSION=$(grep '^version' "$MACROS_DIR/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
MAIN_VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')

info "rok-auth-macros v${MACROS_VERSION}"
info "rok-auth        v${MAIN_VERSION}"

# Confirm versions match
[[ "$MACROS_VERSION" == "$MAIN_VERSION" ]] || \
  warn "version mismatch — macros=${MACROS_VERSION}, main=${MAIN_VERSION}"

# Require clean working tree for a real publish
if [[ "$DRY_RUN" == false ]]; then
  if ! git -C "$ROOT_DIR" diff --quiet HEAD; then
    die "working tree has uncommitted changes — commit everything before publishing"
  fi
fi

# ── Run checks ────────────────────────────────────────────────────────────────
info "Running cargo fmt --check …"
cargo fmt --manifest-path "$ROOT_DIR/Cargo.toml" --check \
  || die "formatting issues found — run 'cargo fmt' and commit"

info "Running cargo clippy …"
cargo clippy --manifest-path "$ROOT_DIR/Cargo.toml" -- -D warnings \
  || die "clippy errors — fix them before publishing"

info "Running cargo test …"
cargo test --manifest-path "$ROOT_DIR/Cargo.toml" \
  || die "tests failed — fix them before publishing"

success "All checks passed"

# ── Dry-run both crates ───────────────────────────────────────────────────────
info "Dry-run: rok-auth-macros …"
cargo publish --dry-run --manifest-path "$MACROS_DIR/Cargo.toml" \
  || die "dry-run failed for rok-auth-macros"
success "rok-auth-macros dry-run OK"

info "Dry-run: rok-auth (requires macros already on crates.io) …"
# rok-auth dry-run only succeeds once rok-auth-macros is live on crates.io.
# We skip it here if publishing for the first time.
if cargo publish --dry-run --manifest-path "$ROOT_DIR/Cargo.toml" 2>/dev/null; then
  success "rok-auth dry-run OK"
else
  warn "rok-auth dry-run skipped (rok-auth-macros not yet indexed — normal on first publish)"
fi

# ── Publish ───────────────────────────────────────────────────────────────────
if [[ "$DRY_RUN" == true ]]; then
  echo ""
  warn "Dry-run complete. Run '$0 --publish' to actually publish."
  exit 0
fi

echo ""
echo -e "${BOLD}About to publish:${RESET}"
echo "  1. rok-auth-macros v${MACROS_VERSION}"
echo "  2. rok-auth        v${MAIN_VERSION}"
echo ""
read -rp "Continue? [y/N] " confirm
[[ "${confirm,,}" == "y" ]] || { info "Aborted."; exit 0; }

# Step 1: publish macros
info "Publishing rok-auth-macros v${MACROS_VERSION} …"
cargo publish --manifest-path "$MACROS_DIR/Cargo.toml"
success "rok-auth-macros published"

# Step 2: wait for crates.io to index it
WAIT=30
info "Waiting ${WAIT}s for crates.io to index rok-auth-macros …"
for i in $(seq "$WAIT" -1 1); do
  printf "\r  %2ds remaining …" "$i"
  sleep 1
done
echo ""

# Step 3: poll until the version is actually available
info "Polling crates.io for rok-auth-macros v${MACROS_VERSION} …"
for attempt in $(seq 1 12); do
  if curl -sf "https://crates.io/api/v1/crates/rok-auth-macros/${MACROS_VERSION}" \
       -H "User-Agent: rok-auth publish script" \
       | grep -q '"num":'; then
    success "rok-auth-macros v${MACROS_VERSION} is live on crates.io"
    break
  fi
  if [[ "$attempt" -eq 12 ]]; then
    die "rok-auth-macros not visible on crates.io after 2 minutes — try publishing rok-auth manually"
  fi
  warn "Not indexed yet, waiting 10s (attempt ${attempt}/12) …"
  sleep 10
done

# Step 4: publish main crate
info "Publishing rok-auth v${MAIN_VERSION} …"
cargo publish --manifest-path "$ROOT_DIR/Cargo.toml"
success "rok-auth v${MAIN_VERSION} published!"

echo ""
echo -e "${GREEN}${BOLD}Done.${RESET}"
echo "  https://crates.io/crates/rok-auth-macros"
echo "  https://crates.io/crates/rok-auth"
