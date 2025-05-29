from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
import matplotlib.pyplot as plt
from cycler import cycler
import numpy as np
from utils import read_json_file

def plot_performance(df, name):
    df['x'] = df['AST total nodes']
    df['y'] = df['Time (ms)']

    fig, ax = plt.subplots()

    # Plot the points
    ax.scatter(df['x'], df['y'], color='blue', s=30)
    ax.errorbar(df['x'], df['y'],
                yerr=[df['y'] - df['ci_lower'], df['ci_upper'] - df['y']],
                fmt='none', ecolor='gray', elinewidth=1, capsize=3, alpha=0.7)

    # Labels and title
    plt.title(name)
    plt.xlabel('AST total nodes')
    plt.ylabel('Time (ms)')

    plt.xscale('log')
    plt.tight_layout()
    plt.savefig(name + ".png")

def plot_comparison(title, x, y, datasets, n_bins=None):
    x_for_file = {d['file_name']: d[x] for _, d in datasets[0][0].iterrows()}

    plt.clf()
    plt.rc('axes', prop_cycle=cycler('color', plt.cm.tab10.colors))

    for (df, label) in datasets:
        df = df[df['file_name'].isin(x_for_file)]
        df['x'] = df['file_name'].map(x_for_file)
        df['y'] = df[y]

        if n_bins is not None:
            bins = np.logspace(np.log10(df['x'].min()), np.log10(df['x'].max()), n_bins)
            df['bin'] = pd.cut(df['x'], bins=bins)
            df = df.groupby('bin').agg(
                y=('y', 'mean'),
                x=('x', 'mean')  # or: lambda x: x.mean(), or use b.mid below
            ).reset_index()

        plt.plot(df['x'], df['y'], marker='o', linestyle=('-' if n_bins is not None else ''), label=label)

    plt.xlabel(x)
    plt.ylabel(y)
    plt.xscale('log')
    plt.legend()
    plt.title(title)
    plt.grid(True)
    plt.savefig(title + ".png")

greedy_1000_data = read_json_file("../results_greedy_1000.json")
xy_data = read_json_file("../results_xy.json")
greedy_100_data = read_json_file("../results_greedy_100.json")
greedy_300_data = read_json_file("../results_greedy_300.json")
greedy_500_data = read_json_file("../results_greedy_500.json")

plot_performance(greedy_1000_data, "Greedy S=1000 performance")
plot_performance(greedy_100_data, "Greedy S=100 performance")
plot_performance(greedy_300_data, "Greedy S=300 performance")
plot_performance(greedy_500_data, "Greedy S=500 performance")
plot_performance(xy_data, "XY performance")

plot_comparison(
    title = "Quality (matched nodes)",
    x = 'Best algorithm matched nodes',
    y = 'Matched nodes',
    datasets = [(greedy_1000_data, 'GreedyBottomUpMatcher, S=1000'),
                (greedy_500_data, 'GreedyBottomUpMatcher, S=500'),
                (greedy_300_data, 'GreedyBottomUpMatcher, S=300'),
                (greedy_100_data, 'GreedyBottomUpMatcher, S=100'),
                (xy_data, 'XYMatcher')],
)

plot_comparison(
    title = "Quality by algorithm",
    x = 'Best algorithm matched nodes',
    y = 'Matched nodes',
    datasets = [(greedy_1000_data, 'GreedyBottomUpMatcher, S=1000'),
                (greedy_500_data, 'GreedyBottomUpMatcher, S=500'),
                (greedy_300_data, 'GreedyBottomUpMatcher, S=300'),
                (greedy_100_data, 'GreedyBottomUpMatcher, S=100'),
                (xy_data, 'XYMatcher')],
    n_bins = 20
)

plot_comparison(
    title = "Performance by algorithm",
    x = 'Best algorithm matched nodes',
    y = 'Time (ms)',
    datasets = [(greedy_1000_data, 'GreedyBottomUpMatcher, S=1000'),
                (greedy_500_data, 'GreedyBottomUpMatcher, S=500'),
                (greedy_300_data, 'GreedyBottomUpMatcher, S=300'),
                (greedy_100_data, 'GreedyBottomUpMatcher, S=100'),
                (xy_data, 'XYMatcher')],
    n_bins = 20
)