from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
import matplotlib.pyplot as plt
from cycler import cycler
import numpy as np
from utils import read_json_file
import seaborn as sns

def plot_performance(df, name, x='AST total nodes'):
    df['x'] = df[x]
    df['y'] = df['Time (ms)']

    fig, ax = plt.subplots()

    # Plot the points
    ax.scatter(df['x'], df['y'], color='blue', s=30)
    ax.errorbar(df['x'], df['y'],
                yerr=[df['y'] - df['ci_lower'], df['ci_upper'] - df['y']],
                fmt='none', ecolor='gray', elinewidth=1, capsize=3, alpha=0.7)

    # Labels and title
    plt.title(name)
    plt.xlabel(x)
    plt.ylabel('Time (ms)')

    plt.xscale('log')
    plt.yscale('log')
    plt.tight_layout()
    plt.savefig(name + ".png")

def plot_comparison(title, x, y, datasets):
    x_for_file = {d['file_name']: d[x] for _, d in datasets[0][0].iterrows()}

    plt.clf()
    plt.rc('axes', prop_cycle=cycler('color', plt.cm.tab10.colors))

    for (df, label) in datasets:
        df = df[df['file_name'].isin(x_for_file)]
        df['x'] = df['file_name'].map(x_for_file)
        df['y'] = df[y]

        plt.plot(df['x'], df['y'], marker='x', linestyle='', label=label)

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
    bins = np.logspace(np.log10(min(x_for_file.values())), np.log10(max(x_for_file.values())), n_bins + 2)
    bins[-1] += 1
    bins = bins[1:]

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
    sns.stripplot(data=plot_df, x='bin', y='y', hue='label', jitter=0.5, dodge=True, alpha=0.7)
    plt.xticks(
        ticks=np.arange(n_bins),
        labels=[f"{bins[i]:.0f} - {bins[i+1]:.0f}" for i in range(n_bins)],
        rotation=45,
        ha='right'
    )

    plt.xlabel(x)
    plt.ylabel(y)
    plt.yscale('log')
    plt.legend()
    plt.title(title)
    plt.tight_layout()
    plt.savefig(title + ".png")

xy_data = read_json_file("../results_xy.json")
greedy_100_data = read_json_file("../results_greedy_100.json")
greedy_300_data = read_json_file("../results_greedy_300.json")
#greedy_500_data = read_json_file("../results_greedy_500.json")
greedy_1000_data = read_json_file("../results_greedy_1000.json")
simple_data = read_json_file("../results_simple.json")
gumtreediff_1000_data = read_json_file("../results_gumtreediff.json")
hyperdiff_1000_data = read_json_file("../results_hyperdiff.json")

plot_performance(greedy_1000_data, "Greedy S=1000 performance", x='Matched nodes')
plot_performance(greedy_100_data, "Greedy S=100 performance")
plot_performance(greedy_300_data, "Greedy S=300 performance")
#plot_performance(greedy_500_data, "Greedy S=500 performance")
plot_performance(xy_data, "XY performance")
plot_performance(simple_data, "Simple performance")

datasets_bottomup = [(greedy_1000_data, 'Greedy Matcher, S=1000'),
                    #(greedy_500_data, 'Greedy Matcher, S=500'),
                    (greedy_300_data, 'Greedy Matcher, S=300'),
                    (greedy_100_data, 'Greedy Matcher, S=100'),
                    (xy_data, 'XYMatcher'),
                    (simple_data, 'Simple Matcher')]
plot_comparison(
    title = "Quality (raw)",
    x = 'Best algorithm matched nodes',
    y = 'Script length difference',
    datasets = datasets_bottomup,
)

plot_comparison_binned(
    title = "Quality by algorithm (total nodes)",
    x = 'AST total nodes',
    y = 'Matched nodes',
    datasets = datasets_bottomup
)

plot_comparison_binned(
    title = "Runtime by algorithm (matched nodes)",
    x = 'Best algorithm matched nodes',
    y = 'Time (ms)',
    datasets = datasets_bottomup
)

plot_comparison_binned(
    title = "Runtime by algorithm (total size)",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = datasets_bottomup
)

dataset_diff = [(gumtreediff_1000_data, 'GumtreeDiff'),
                (hyperdiff_1000_data, 'HyperDiff')]

plot_comparison(
    title = "Diff Runtime (raw)",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = dataset_diff,
)

plot_comparison_binned(
    title = "Diff Runtime by algorithm (total size)",
    x = 'AST total nodes',
    y = 'Time (ms)',
    datasets = dataset_diff
)