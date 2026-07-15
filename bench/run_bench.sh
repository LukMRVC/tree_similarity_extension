#!/usr/bin/env bash
#
# Pipeline-search benchmark: every pipeline method x every dataset.
#
# Measures wall-clock time of each of the 100 queries per (dataset, method)
# pair against the full tree collection, timed server-side.
#
# The run is resumable. results.csv is the checkpoint: a query whose row is
# already there with status=ok is skipped on the next invocation, so the script
# can be killed and restarted at any point. Rows with status=error are retried.
#
#   ./bench/run_bench.sh                      # build, load, run everything
#   ./bench/run_bench.sh --skip-build         # reuse the installed extension
#   ./bench/run_bench.sh --only-dataset rna   # repeatable; also --only-method
#   ./bench/run_bench.sh --load-only          # populate the DBs, run nothing
#
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
PG_CONFIG="${PG_CONFIG:-$HOME/.pgrx/18.4/pgrx-install/bin/pg_config}"
PG_VER="${PG_VER:-pg18}"
PGBIN="$(dirname "$PG_CONFIG")"
export PGHOST="${PGHOST:-localhost}"
export PGPORT="${PGPORT:-28818}"
DB="${DB:-tree_bench}"

BRACKET_PREP="${BRACKET_PREP:-$REPO/../ted-search/target/release/bracket-prep}"
DATA_DIR="${DATA_DIR:-$REPO/datasets}"
OUT_DIR="${OUT_DIR:-$REPO/bench/results}"
RESULTS="$OUT_DIR/results.csv"
LOG="$OUT_DIR/run.log"

# Dataset order is the requested one; method order does not matter.
DATASETS=(sentiment rna ptb treefam dblp python swissprot)
METHODS=(
  sed_topdiff_within
  sed_plain_topdiff_within
  structural_topdiff_within
  binary_branch_topdiff_within
  lblint_topdiff_within
)

SKIP_BUILD=0
LOAD_ONLY=0
ONLY_DATASETS=()
ONLY_METHODS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build)    SKIP_BUILD=1; shift ;;
    --load-only)     LOAD_ONLY=1; shift ;;
    --only-dataset)  ONLY_DATASETS+=("$2"); shift 2 ;;
    --only-method)   ONLY_METHODS+=("$2"); shift 2 ;;
    -h|--help)       sed -n '2,18p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ ${#ONLY_DATASETS[@]} -gt 0 ]] && DATASETS=("${ONLY_DATASETS[@]}")
[[ ${#ONLY_METHODS[@]}  -gt 0 ]] && METHODS=("${ONLY_METHODS[@]}")

mkdir -p "$OUT_DIR"

log() { printf '%s  %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$LOG"; }

# Scalar query against the benchmark DB.
psql_q() { "$PGBIN/psql" -d "$DB" -X -q -A -t -F',' -v ON_ERROR_STOP=1 -c "$1"; }

# ---------------------------------------------------------------------------
# Phase 1 - build + install the extension (release)
# ---------------------------------------------------------------------------
if [[ $SKIP_BUILD -eq 0 ]]; then
  log "building extension (release) -> $PG_VER"
  cargo pgrx install --release --pg-config "$PG_CONFIG" >>"$LOG" 2>&1
  log "build ok"
else
  log "skipping build"
fi

# ---------------------------------------------------------------------------
# Phase 2 - server up, database ready
# ---------------------------------------------------------------------------
ensure_server() {
  if ! "$PGBIN/pg_isready" -q 2>/dev/null; then
    log "postgres not ready, starting $PG_VER"
    cargo pgrx start "$PG_VER" >>"$LOG" 2>&1 || true
    for _ in $(seq 1 30); do
      "$PGBIN/pg_isready" -q 2>/dev/null && break
      sleep 1
    done
  fi
  "$PGBIN/pg_isready" -q
}

ensure_server
"$PGBIN/psql" -d postgres -X -q -tAc \
  "SELECT 1 FROM pg_database WHERE datname='$DB'" | grep -q 1 \
  || "$PGBIN/createdb" "$DB"

psql_q "CREATE EXTENSION IF NOT EXISTS tree_similarity_extension;" >/dev/null

psql_q "
CREATE TABLE IF NOT EXISTS bench_loaded(
  dataset   text        NOT NULL,
  kind      text        NOT NULL,
  n_rows    bigint      NOT NULL,
  loaded_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (dataset, kind)
);" >/dev/null

# Times the scan server-side, so connection and client overhead stay out of the
# number. Table and function names are identifiers, hence format/%I rather than
# string concatenation.
psql_q "
CREATE OR REPLACE FUNCTION bench_one(p_ds text, p_method text, p_qid int)
RETURNS TABLE(matches bigint, elapsed_ms double precision)
LANGUAGE plpgsql AS \$\$
DECLARE
  t0 timestamptz;
  t1 timestamptz;
  n  bigint;
BEGIN
  t0 := clock_timestamp();
  EXECUTE format(
    'SELECT count(*) FROM trees_%I t, queries_%I q
      WHERE q.id = \$1 AND %I(q.tree, t.tree, q.k) <= q.k',
    p_ds, p_ds, p_method
  ) INTO n USING p_qid;
  t1 := clock_timestamp();
  matches    := n;
  elapsed_ms := extract(epoch FROM (t1 - t0)) * 1000.0;
  RETURN NEXT;
END \$\$;" >/dev/null

# ---------------------------------------------------------------------------
# Phase 3 - load trees + queries
#
# bracket-prep keeps only the lines the ted-search reference loader keeps and
# emits COPY-TEXT rows; ids are original file line numbers, so dropped lines
# never shift the ids of the survivors. A load is recorded in bench_loaded only
# after it succeeds, so an interrupted load is simply redone.
# ---------------------------------------------------------------------------
[[ -x "$BRACKET_PREP" ]] || {
  echo "bracket-prep not found at $BRACKET_PREP" >&2
  echo "build it: (cd ../ted-search && cargo build --release -p bracket-prep)" >&2
  exit 1
}

load_kind() {
  local ds=$1 kind=$2 file=$3 table=$4 cols=$5
  local want
  want=$(psql_q "SELECT n_rows FROM bench_loaded WHERE dataset='$ds' AND kind='$kind'")
  if [[ -n "$want" ]]; then
    log "  $ds/$kind already loaded ($want rows)"
    return 0
  fi

  log "  loading $ds/$kind"
  psql_q "DROP TABLE IF EXISTS $table;" >/dev/null
  psql_q "CREATE TABLE $table($cols);" >/dev/null
  "$BRACKET_PREP" "$kind" "$file" 2>>"$LOG" \
    | "$PGBIN/psql" -d "$DB" -X -q -v ON_ERROR_STOP=1 \
        -c "COPY $table FROM STDIN" >>"$LOG" 2>&1

  local n
  n=$(psql_q "SELECT count(*) FROM $table")
  psql_q "INSERT INTO bench_loaded(dataset,kind,n_rows) VALUES('$ds','$kind',$n)
          ON CONFLICT (dataset,kind) DO UPDATE SET n_rows=EXCLUDED.n_rows, loaded_at=now();" >/dev/null
  log "  $ds/$kind loaded: $n rows"
}

log "=== load phase ==="
for ds in "${DATASETS[@]}"; do
  load_kind "$ds" trees   "$DATA_DIR/$ds/trees_sorted.bracket" "trees_$ds"   "id int PRIMARY KEY, tree unifiedtreeindex"
  load_kind "$ds" queries "$DATA_DIR/$ds/query.csv"            "queries_$ds" "id int PRIMARY KEY, k int NOT NULL, tree unifiedtreeindex"
done

if [[ $LOAD_ONLY -eq 1 ]]; then
  log "load-only requested, stopping"
  exit 0
fi

# ---------------------------------------------------------------------------
# Phase 4 - run the matrix
# ---------------------------------------------------------------------------
[[ -f "$RESULTS" ]] || echo "dataset,method,query_id,k,matches,elapsed_ms,status,ts" > "$RESULTS"

# Resume set: completed queries only. status=error rows are retried and append a
# fresh row, so take the last row per key when analysing.
declare -A DONE=()
while IFS=, read -r r_ds r_m r_qid _r_k _r_n _r_ms r_status _r_ts; do
  [[ "$r_status" == "ok" ]] && DONE["$r_ds,$r_m,$r_qid"]=1
done < <(tail -n +2 "$RESULTS")
log "resume: ${#DONE[@]} queries already completed"

log "=== run phase ==="
for ds in "${DATASETS[@]}"; do
  mapfile -t QIDS < <(psql_q "SELECT id FROM queries_$ds ORDER BY id")
  for m in "${METHODS[@]}"; do
    pair_t0=$(date +%s)
    ran=0
    for qid in "${QIDS[@]}"; do
      [[ -n "${DONE[$ds,$m,$qid]:-}" ]] && continue
      ensure_server || { log "server down and will not start; aborting"; exit 1; }

      k=$(psql_q "SELECT k FROM queries_$ds WHERE id=$qid")
      ts=$(date -Iseconds)
      if row=$(psql_q "SELECT matches, round(elapsed_ms::numeric,3) FROM bench_one('$ds','$m',$qid)" 2>>"$LOG"); then
        echo "$ds,$m,$qid,$k,$row,ok,$ts" >> "$RESULTS"
      else
        log "  ERROR $ds/$m/q$qid (see $LOG)"
        echo "$ds,$m,$qid,$k,,,error,$ts" >> "$RESULTS"
      fi
      ran=$((ran + 1))
    done
    if [[ $ran -gt 0 ]]; then
      log "$ds / $m: $ran queries in $(( $(date +%s) - pair_t0 ))s"
    else
      log "$ds / $m: already complete, skipped"
    fi
  done
done

log "=== done -> $RESULTS ==="
summarise() {
  # Last row wins per (dataset,method,query_id), matching the resume rule above.
  awk -F, 'NR>1 { key=$1","$2","$3; status[key]=$7; ms[key]=$6; pair[key]=$1","$2 }
  END {
    for (k in status) {
      p = pair[k]
      if (status[k] == "ok") { n[p]++; tot[p] += ms[k] } else { err[p]++ }
    }
    printf "%-11s %-30s %7s %7s %12s %10s\n", "dataset", "method", "queries", "errors", "total_ms", "avg_ms"
    for (p in n) {
      split(p, a, ",")
      printf "%-11s %-30s %7d %7d %12.1f %10.1f\n", a[1], a[2], n[p], err[p]+0, tot[p], tot[p]/n[p]
    }
  }' "$RESULTS" | { read -r hdr; echo "$hdr"; sort; }
}
summarise | tee -a "$LOG"
