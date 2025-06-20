from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
from sklearn.linear_model import LinearRegression
import matplotlib.pyplot as plt

from utils import read_json_file

def correlations(df, column_labels):
    return [df[column_label].corr(df['Time (ms)']).round(3) for column_label in column_labels]

hyperdiff_data = read_json_file("../results_hyperdiff.json")
gumtreediff_data = read_json_file("../results_gumtreediff.json")

row_labels = ['HyperDiff', 'GumTreeDiff']
column_labels = ['AST total nodes', 'Unmatched nodes (Subtree)', 'Unmatched nodes (BottomUp)', 'Matched nodes']

values = [
    correlations(hyperdiff_data, column_labels),
    correlations(gumtreediff_data, column_labels),
]

plt.figure(figsize=(10, 6))
plt.table(cellText=values, rowLabels=row_labels, colLabels=column_labels, loc='center')
plt.axis('off')
plt.title("Pearson correlation between different variables and performance")
plt.show()