from plotnine import ggplot, aes, geom_point, geom_errorbar, labs, theme_minimal
import pandas as pd
import json
from sklearn.linear_model import LinearRegression
import matplotlib.pyplot as plt
from utils import read_json_file

def correlations(df, column_labels):
    return [df[column_label].corr(df['Time (ms)']).round(3) for column_label in column_labels]

greedy_500_data = read_json_file("../results_greedy_500.json")
xy_data = read_json_file("../results_xy.json")
greedy_100_data = read_json_file("../results_greedy_100.json")

row_labels = ['Greedy 500','Greedy 100', 'XY']
column_labels = ['AST total nodes', 'Unmatched before', 'Unmatched after', 'Matched nodes']

values = [
    correlations(greedy_500_data, column_labels),
    correlations(greedy_100_data, column_labels),
    correlations(xy_data, column_labels),
]

plt.figure()
plt.table(cellText=values, rowLabels=row_labels, colLabels=column_labels, loc='center')
plt.axis('off')
plt.title("Pearson correlation between different variables and performance")
plt.show()