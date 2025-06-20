from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
import numpy as np
import seaborn as sns
from matplotlib.lines import Line2D
import matplotlib.pyplot as plt

def read_json_file(file_path):
    with open(file_path, 'r') as f:
        data = json.load(f)
        df = pd.DataFrame([{
            'file_name': d['file_name'],
            'AST total nodes': d['size'],
            "matches_before": d['matches_before'],
            "matches_after": d['matches_after'],
            "Script length before": d['script_length_before'],
            "Script length after": d['script_length_after'],
            "Script length reduction": d['script_length_before'] - d['script_length_after'],
            'Unmatched nodes (Subtree)': d['size'] - d['matches_before'] * 2,
            'Unmatched nodes (BottomUp)': d['size'] - d['matches_after'] * 2,
            'Matched nodes': d['matches_after'] - d['matches_before'],
            'Time (ms)': d['criterion']['mean']['point_estimate'] / 1000000.0,
            'ci_lower': d['criterion']['mean']['confidence_interval']['lower_bound'] / 1000000.0,
            'ci_upper': d['criterion']['mean']['confidence_interval']['upper_bound'] / 1000000.0
        } for d in data])
        return df

def add_markers(ax, dataset_count, plot_df):
    markers = ['+', 'x', '.', '^', 's']
    colors = sns.color_palette("deep")

    collections = ax.collections  # List of PathCollections (one per hue level)

    for i, coll in enumerate(collections):
        index = i % dataset_count
        coll.set_facecolor('none')
        coll.set_edgecolor(colors[index])
        coll.set_linewidth(1)
        if index >= 3:
            coll.set_sizes([10])
        coll.set_paths([plt.matplotlib.markers.MarkerStyle(markers[index]).get_path().transformed(
            plt.matplotlib.markers.MarkerStyle(markers[index]).get_transform())])  # Set marker style

    legend_elements = [
        Line2D(
            [0], [0],
            marker=markers[i],
            color='none',
            markerfacecolor='none',
            markeredgecolor=colors[i],
            markeredgewidth=1,
            markersize=6,
            linestyle='None',
            label=label
        )
        for i, label in enumerate(plot_df['label'].unique())
    ]
    ax.legend(handles=legend_elements)