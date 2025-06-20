from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
import matplotlib.pyplot as plt
from cycler import cycler
import numpy as np
from utils import read_json_file, add_markers
import seaborn as sns

def plot_performance(df, name, x='AST total nodes'):
    df = df[df[x] > 1]
    df['x'] = df[x] + np.random.uniform(-0.5, 0.5, size=len(df))
    df['y'] = df['Time (ms)']

    fig, ax = plt.subplots(figsize=(10, 6))

    ax.scatter(df['x'], df['y'], color='blue', s=30, marker='x')
    ax.errorbar(df['x'], df['y'],
                yerr=[df['y'] - df['ci_lower'], df['ci_upper'] - df['y']],
                fmt='none', ecolor='gray', elinewidth=1, capsize=3, alpha=0.7)

    plt.title(name)
    plt.xlabel(x)
    plt.ylabel('Time (ms)')

    plt.xscale('log')
    plt.yscale('log')
    plt.tight_layout()
    plt.savefig(name + " (" + x + ").png")

def plot_comparison(title, x, y, datasets, s=None):
    x_for_file = {d['file_name']: d[x] for _, d in datasets[0][0].iterrows()}

    plt.clf()
    plt.rc('axes', prop_cycle=cycler('color', plt.cm.tab10.colors))

    for (df, label) in datasets:
        df = df[df['file_name'].isin(x_for_file)]
        df['x'] = df['file_name'].map(x_for_file)
        df['y'] = df[y]
        if s:
            df['size'] = np.sqrt(df[s]) * 5

        plt.scatter(df['x'], df['y'], s=df['size'] if s else None, marker='x', linestyle='', label=label)

    plt.xlabel(x)
    plt.ylabel(y)
    plt.xscale('log')
    plt.yscale('log')
    plt.legend()
    plt.title(title)
    plt.tight_layout()
    plt.savefig(title + ".png")
def plot_comparison_binned(title, x, y, datasets, n_bins=12):
    x_for_file = {d['file_name']: d[x] for _, d in datasets[0][0].iterrows() if d[x] > 1}

    all_data = []

    # set up bins
    bins = np.logspace(np.log10(min(x_for_file.values())), np.log10(max(x_for_file.values())), n_bins + 2)
    bins[-1] += 1
    bins = bins[1:]
    while np.count_nonzero(list(x_for_file.values()) > bins[-2]) < 10:
        bins = np.delete(bins, -2)
    bins = np.round(bins, -1 if bins[-2] < 10000 else -2)
    bins = np.unique(bins)

    n_bins = len(bins) - 1

    for (df, label) in datasets:
        df = df[df['file_name'].isin(x_for_file)]
        df['x'] = df['file_name'].map(x_for_file)
        df['y'] = df[y]

        df = df.copy()
        df['bin'] = pd.cut(df['x'], bins=bins, labels=False, include_lowest=True, right=False)

        df['label'] = label
        all_data.append(df[['bin', 'y', 'label']])

    plot_df = pd.concat(all_data, ignore_index=True)
    plot_df = plot_df.dropna(subset=['bin'])
    plot_df['bin'] = plot_df['bin'].astype(int)

    plt.figure(figsize=(10, 6))
    ax = sns.stripplot(data=plot_df, x='bin', y='y', hue='label', jitter=0.5, dodge=True, alpha=0.7)

    ax.set_xlim(-0.5, n_bins - 0.5)

    add_markers(ax, len(datasets), plot_df)

    plt.xticks(
        ticks=np.arange(n_bins),
        labels=[f"{bins[i]:.0f} - {bins[i+1]:.0f}" for i in range(n_bins)],
        rotation=45,
        ha='right'
    )

    plt.xlabel(x)
    plt.ylabel(y)
    plt.yscale('log')
    plt.title(title)
    plt.tight_layout()
    plt.savefig(title + " (" + x + ").png", dpi=300)

xy_data = read_json_file("../results_xy.json")
greedy_100_data = read_json_file("../results_greedy_100.json")
greedy_300_data = read_json_file("../results_greedy_300.json")
greedy_1000_data = read_json_file("../results_greedy_1000.json")
simple_data = read_json_file("../results_simple.json")
gumtreediff_data = read_json_file("../results_gumtreediff.json")
hyperdiff_data = read_json_file("../results_hyperdiff.json")

plot_performance(greedy_1000_data, "GumTreeBottomUp MAX_SIZE=1000")
plot_performance(greedy_100_data, "GumTreeBottomUp MAX_SIZE=100", x='AST total nodes')
plot_performance(greedy_100_data, "GumTreeBottomUp MAX_SIZE=100", x='Matched nodes')
plot_performance(greedy_300_data, "GumTreeBottomUp MAX_SIZE=300")
plot_performance(xy_data, "XY performance")
plot_performance(simple_data, "Simple performance")

datasets_bottomup = [(greedy_1000_data, 'GumTreeBottomUp, MAX_SIZE=1000'),
                    (greedy_300_data, 'GumTreeBottomUp, MAX_SIZE=300'),
                    (greedy_100_data, 'GumTreeBottomUp, MAX_SIZE=100'),
                    (xy_data, 'XyBottomUp'),
                    (simple_data, 'SimpleBottomUp'),]
plot_comparison(
    title = "BottomUp Quality (raw)",
    x = 'Matched nodes',
    y = 'Script length reduction',
    datasets = datasets_bottomup,
)

plot_comparison_binned(
    title = "BottomUp Quality",
    x = 'AST total nodes',
    y = 'Script length reduction',
    datasets = datasets_bottomup
)

plot_comparison_binned(
    title = "BottomUp Runtime",
    x = 'Matched nodes',
    y = 'Time (ms)',
    datasets = datasets_bottomup
)

plot_comparison_binned(
    title = "BottomUp Runtime",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = datasets_bottomup
)

dataset_diff = [(gumtreediff_data, 'GumTreeSubtree'),
                (hyperdiff_data, 'HyperDiffSubtree')]

plot_comparison(
    title = "Runtime (raw)",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = dataset_diff,
)

plot_comparison_binned(
    title = "Subtree Runtime",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = dataset_diff
)

plot_comparison_binned(
    title = "Subtree Runtime",
    x = 'Unmatched nodes (Subtree)',
    y = 'Time (ms)',
    datasets = dataset_diff
)