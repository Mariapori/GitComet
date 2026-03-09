#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

script_name="$(basename "$0")"

usage() {
  cat <<EOF
Usage: ${script_name} [options]

Run Codex in a loop to improve line coverage until a target percentage is reached
or max iterations is hit.

Options:
  --target <percent>          Target total line coverage percentage (default: 80)
  --max-iterations <count>    Maximum Codex iterations (default: 8)
  --coverage-cmd <command>    Coverage command to execute (default: "bash scripts/coverage.sh")
  --lcov-file <path>          LCOV file to parse after coverage command (default: target/llvm-cov/lcov.info)
  --codex-bin <path>          Codex executable (default: codex)
  --codex-arg <arg>           Extra argument to pass to "codex exec" (repeatable)
  --log-dir <path>            Directory for iteration logs (default: target/codex-coverage-loop)
  -h, --help                  Show this help

Examples:
  ${script_name}
  ${script_name} --target 85 --max-iterations 12
  ${script_name} --codex-arg "--model" --codex-arg "gpt-5"
EOF
}

target_coverage="80"
max_iterations="8"
coverage_cmd="bash scripts/coverage.sh"
lcov_file="target/llvm-cov/lcov.info"
codex_bin="codex"
log_dir="target/codex-coverage-loop"
codex_args=(--full-auto)

while (($# > 0)); do
  case "$1" in
    --target)
      target_coverage="${2:-}"
      shift 2
      ;;
    --max-iterations)
      max_iterations="${2:-}"
      shift 2
      ;;
    --coverage-cmd)
      coverage_cmd="${2:-}"
      shift 2
      ;;
    --lcov-file)
      lcov_file="${2:-}"
      shift 2
      ;;
    --codex-bin)
      codex_bin="${2:-}"
      shift 2
      ;;
    --codex-arg)
      codex_args+=("${2:-}")
      shift 2
      ;;
    --log-dir)
      log_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$target_coverage" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "--target must be numeric, got: $target_coverage" >&2
  exit 1
fi

if ! [[ "$max_iterations" =~ ^[0-9]+$ ]] || [[ "$max_iterations" == "0" ]]; then
  echo "--max-iterations must be a positive integer, got: $max_iterations" >&2
  exit 1
fi

if ! command -v "$codex_bin" >/dev/null 2>&1; then
  echo "Codex executable not found: $codex_bin" >&2
  exit 1
fi

mkdir -p "$log_dir"

float_ge() {
  awk -v left="$1" -v right="$2" 'BEGIN { exit !(left + 0 >= right + 0) }'
}

float_gt() {
  awk -v left="$1" -v right="$2" 'BEGIN { exit !(left + 0 > right + 0) }'
}

float_sub() {
  awk -v left="$1" -v right="$2" 'BEGIN { printf "%.2f", left - right }'
}

parse_lcov_line_coverage() {
  local file="$1"
  awk -F '[:,]' '
    /^DA:/ {
      total += 1
      if ($3 > 0) {
        covered += 1
      }
    }
    END {
      if (total == 0) {
        exit 1
      }
      printf "%.2f\n", (covered / total) * 100
    }
  ' "$file"
}

parse_summary_coverage_from_log() {
  local file="$1"
  awk '
    /TOTAL/ {
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^[0-9]+([.][0-9]+)?%$/) {
          value = $i
        }
      }
    }
    END {
      if (value == "") {
        exit 1
      }
      gsub("%", "", value)
      print value
    }
  ' "$file"
}

measure_coverage() {
  local iteration="$1"
  local coverage_log="${log_dir}/coverage-${iteration}.log"
  local coverage_value

  echo "Running coverage command..."
  if ! bash -lc "$coverage_cmd" | tee "$coverage_log"; then
    echo "Coverage command failed. See: $coverage_log" >&2
    return 1
  fi

  if [[ -f "$lcov_file" ]]; then
    coverage_value="$(parse_lcov_line_coverage "$lcov_file")"
  else
    coverage_value="$(parse_summary_coverage_from_log "$coverage_log")"
  fi

  echo "$coverage_value"
}

run_codex_iteration() {
  local iteration="$1"
  local current_coverage="$2"
  local codex_log="${log_dir}/codex-${iteration}.log"
  local prompt_file="${log_dir}/prompt-${iteration}.txt"

  cat >"$prompt_file" <<EOF
You are working inside the GitComet repository.

Task: increase application line coverage beyond ${current_coverage}% toward ${target_coverage}%.
Iteration: ${iteration}/${max_iterations}.

Allowed strategies (choose the highest impact):
1. Add or improve tests.
2. Simplify code that is difficult to test while preserving behavior.
3. Remove dead code or reduce duplication that hurts coverage and maintainability.

Constraints:
- Preserve behavior and keep the project buildable.
- Do not commit changes.
- Keep changes focused and reviewable.
- Run relevant tests locally for the areas you touch.

Before finishing, provide:
- a concise list of changed files
- why the change should increase coverage
- any risk or follow-up items
EOF

  echo "Running Codex iteration ${iteration}/${max_iterations}..."
  if ! "$codex_bin" exec "${codex_args[@]}" -C "$repo_root" - <"$prompt_file" | tee "$codex_log"; then
    echo "Codex iteration ${iteration} failed. See: $codex_log" >&2
    return 1
  fi
}

echo "Repo root: $repo_root"
echo "Target coverage: ${target_coverage}%"
echo "Max iterations: $max_iterations"
echo "Coverage command: $coverage_cmd"
echo "LCOV file: $lcov_file"
echo "Codex command: $codex_bin exec ${codex_args[*]}"
echo "Log directory: $log_dir"

current_coverage="$(measure_coverage baseline)"
best_coverage="$current_coverage"

echo "Baseline coverage: ${current_coverage}%"
if float_ge "$current_coverage" "$target_coverage"; then
  echo "Target already reached. Nothing to do."
  exit 0
fi

for ((iteration = 1; iteration <= max_iterations; iteration++)); do
  run_codex_iteration "$iteration" "$current_coverage"

  new_coverage="$(measure_coverage "$iteration")"
  delta="$(float_sub "$new_coverage" "$current_coverage")"

  echo "Coverage after iteration ${iteration}: ${new_coverage}% (delta ${delta}%)"

  if float_gt "$new_coverage" "$best_coverage"; then
    best_coverage="$new_coverage"
  fi

  current_coverage="$new_coverage"

  if float_ge "$current_coverage" "$target_coverage"; then
    echo "Target reached at iteration ${iteration}: ${current_coverage}%"
    exit 0
  fi
done

echo "Reached max iterations."
echo "Best coverage achieved: ${best_coverage}% (target ${target_coverage}%)"
exit 2
