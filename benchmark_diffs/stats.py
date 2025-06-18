import pandas as pd
from scipy.stats import wilcoxon, rankdata

# Load the CSV
df = pd.read_csv("benchmark_result_defects4j.csv")

# Drop the first 2 runs per file and variant
df = df[df["run"] > 2]

# Pivot into: (file, variant) → median runtime
pivoted = (
    df.groupby(["file", "variant"])["runtime"]
    .median()
    .unstack()
)

def rank_biserial_correlation(x, y):
    diffs = x - y
    non_zero = diffs[diffs != 0]
    signs = non_zero.apply(lambda d: 1 if d > 0 else -1)
    ranks = rankdata(abs(non_zero), method="average")
    W_plus = sum(r for s, r in zip(signs, ranks) if s > 0)
    W_minus = sum(r for s, r in zip(signs, ranks) if s < 0)
    return (W_plus - W_minus) / (W_plus + W_minus)

comparisons = [
    ("Greedy", "Stable"),
    ("Lazy Greedy", "Lazy Stable"),
    ("Greedy", "Lazy Greedy"),
    ("Stable", "Lazy Stable"),
]

for a, b in comparisons:
    x = pivoted[a].dropna()
    y = pivoted[b].dropna()
    common_index = x.index.intersection(y.index)

    x = x.loc[common_index]
    y = y.loc[common_index]

    if len(x) < 10:
        print(f"Not enough data points for {var_a} vs {var_b}")

    w_stat, p_value = wilcoxon(x, y)
    rbc = rank_biserial_correlation(x, y)

    non_zero_diffs = (x - y)[(x - y) != 0]
    n = len(non_zero_diffs)
    R = n * (n + 1) / 2
    normalized_w = w_stat / R

    print(f"{a} vs {b}")
    print(f"  w-stat: {w_stat:.3e}")
    print(f"  normalized-w: {normalized_w:.3e}")
    print(f"  p-value: {p_value:.3e}")
    print(f"  rank-biserial correlation: {rbc:.3f}\n")
