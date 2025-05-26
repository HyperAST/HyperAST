(find . -type f -path "./target/criterion/optimized_change_distiller/CD Single*/new/estimates.json" | while read estimate_path; do
  dir=$(dirname "$estimate_path")
  benchmark_path="$dir/benchmark.json"

  if [ -f "$benchmark_path" ]; then
    jq -s 'add' "$estimate_path" "$benchmark_path"
  else
    jq '.' "$estimate_path"
  fi
done | jq -s '.' ) > data.json
