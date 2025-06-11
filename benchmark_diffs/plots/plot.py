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
    plt.yscale('log')
    plt.legend()
    plt.title(title)
    plt.grid(True)
    plt.savefig(title + ".png")

xy_data = read_json_file("../results_xy.json")
greedy_100_data = read_json_file("../results_greedy_100.json")
greedy_300_data = read_json_file("../results_greedy_300.json")
#greedy_500_data = read_json_file("../results_greedy_500.json")
greedy_1000_data = read_json_file("../results_greedy_1000.json")
simple_data = read_json_file("../results_simple.json")

plot_performance(greedy_1000_data, "Greedy S=1000 performance")
plot_performance(greedy_100_data, "Greedy S=100 performance")
plot_performance(greedy_300_data, "Greedy S=300 performance")
#plot_performance(greedy_500_data, "Greedy S=500 performance")
plot_performance(xy_data, "XY performance")
plot_performance(simple_data, "Simple performance")

plot_comparison(
    title = "Quality (script length difference)",
    x = 'Best algorithm matched nodes',
    y = 'Script length difference',
    datasets = [(greedy_1000_data, 'Greedy Matcher, S=1000'),
                #(greedy_500_data, 'Greedy Matcher, S=500'),
                (greedy_300_data, 'Greedy Matcher, S=300'),
                (greedy_100_data, 'Greedy Matcher, S=100'),
                (xy_data, 'XYMatcher'),
                (simple_data, 'Simple Matcher')],
)

plot_comparison(
    title = "Quality by algorithm (matched nodes)",
    x = 'Best algorithm matched nodes',
    y = 'Matched nodes',
    datasets = [(greedy_1000_data, 'Greedy Matcher, S=1000'),
                #(greedy_500_data, 'Greedy Matcher, S=500'),
                (greedy_300_data, 'Greedy Matcher, S=300'),
                (greedy_100_data, 'Greedy Matcher, S=100'),
                (xy_data, 'XYMatcher'),
                (simple_data, 'Simple Matcher')],
    n_bins = 20
)

plot_comparison(
    title = "Quality by algorithm (script length difference)",
    x = 'Best algorithm matched nodes',
    y = 'Script length difference',
    datasets = [(greedy_1000_data, 'Greedy Matcher, S=1000'),
                #(greedy_500_data, 'Greedy Matcher, S=500'),
                (greedy_300_data, 'Greedy Matcher, S=300'),
                (greedy_100_data, 'Greedy Matcher, S=100'),
                (xy_data, 'XYMatcher'),
                (simple_data, 'Simple Matcher')],
    n_bins = 20
)

plot_comparison(
    title = "Performance by algorithm",
    x = 'Best algorithm matched nodes',
    y = 'Time (ms)',
    datasets = [(greedy_1000_data, 'Greedy Matcher, S=1000'),
                #(greedy_500_data, 'Greedy Matcher, S=500'),
                (greedy_300_data, 'Greedy Matcher, S=300'),
                (greedy_100_data, 'Greedy Matcher, S=100'),
                (xy_data, 'XYMatcher'),
                (simple_data, 'Simple Matcher')],
    n_bins = 20
)