function _1(md){return(
md`# ChangeDistiller on HyperAST Custom Benchmarks`
)}

function _ss(require){return(
require('simple-statistics')
)}

function _jStat(require){return(
require('jStat')
)}

async function _raw_data(FileAttachment)
{
  const input = await FileAttachment("cd_benchmark_20250619_062955_r1_tc1046_cfg8@2.jsonl").text()
  const lines = input.split("\n").slice(1);
   
  const jsonObjects = [];

  for (const line of lines) {
    if (line.trim() !== '') {
      try {
        const jsonObject = JSON.parse(line);
        jsonObjects.push(jsonObject);
      } catch (error) {
        console.error('Error parsing JSON:', error);
      }
    }
  }

  return jsonObjects;
}


function _groupedData(d3,raw_data){return(
d3.groups(raw_data, d => `${d.file_name} - ${d.config_name}`)
)}

function _agg_data(d3,raw_data,ss)
{
// Helper to remove outliers using IQR
function removeOutliers(values) {
  if (values.length < 4) return values; // Not enough data to filter

  const sorted = [...values].sort((a, b) => a - b);
  const q1 = d3.quantileSorted(sorted, 0.25);
  const q3 = d3.quantileSorted(sorted, 0.75);
  const iqr = q3 - q1;
  const lower = q1 - 1.5 * iqr;
  const upper = q3 + 1.5 * iqr;
  return sorted.filter((v) => v >= lower && v <= upper);
}

// Group the raw data by test_case_index and config_name
const groupedData = d3.groups(
  raw_data,
  (d) => d.test_case_index,
  (d) => d.config_name,
);

// Map over the grouped data to calculate statistics for each group
const aggregatedResults = groupedData
  .map(([test_case_index, configGroups]) => {
    return configGroups.map(([config_name, groups]) => {
      const stats = (name, key) => {
        const vs = groups.map(typeof key == "string" ? (g) => g[key] : key);
        const filtered = removeOutliers(vs);
        const mean = ss.mean(filtered);
        const median = ss.median(filtered);
        const sd = ss.standardDeviation(filtered);

        const s = (k) => `${name}_${k}`;

        return {
          [s("mean")]: mean,
          [s("median")] : median,
          [s("sd")]: sd,
          [s("mean_upper")]: mean + sd,
          [s("mean_lower")]: mean - sd,
        };
      };

      // Return an object with the grouping keys and calculated statistics
      return {
        ...groups[0],
        ...groups[0].diff_summary,
        no_similarity_checks: groups[0].diff_summary.leaves.similarity_checks,
        

        ...stats("duration_wo_similarity", (d) => d.duration_secs * 1000 - d.diff_summary.leaves.similarity_time),
        ...stats("duration", (d) => d.duration_secs * 1_000),
        ...stats("l_duration", (d) => d.diff_summary.leaves.total_time),
        ...stats("bu_duration", (d) => d.diff_summary.bottomup.total_time),
        ...stats(
          "l_no_comp",
          (d) => d.diff_summary.leaves.total_comparisons,
        ),
        ...stats(
          "l_no_comp",
          (d) => d.diff_summary.leaves.exact_matches,
        ),
        ...stats("l_sim_ms", (d) => d.diff_summary.leaves.similarity_time),
        ...stats("l_no_chars_comp", (d) => d.diff_summary.leaves.characters_compared),
        ...stats("l_exact_matches", (d) => d.diff_summary.leaves.exact_matches),
        

        count: groups.length,
      };
    });
  })
  .flat();

return aggregatedResults.sort((a, b) => b.node_count - a.node_count);


}


function _7(ss,agg_data){return(
ss.median(agg_data.map(e => e.diff_summary.leaves.similarity_time / e.duration_mean))
)}

function _8(__query,agg_data,invalidation){return(
__query(agg_data,{from:{table:"agg_data"},sort:[],slice:{to:null,from:null},filter:[],select:{columns:null}},invalidation,"agg_data")
)}

function _data_opt(agg_data){return(
agg_data.filter(e => e.config_name.includes("Optimized"))
)}

function _data_base(agg_data){return(
agg_data.filter(e => e.config_name.includes("Baseline"))
)}

function _11(Plot,agg_data){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.dot(agg_data, {
      x: "loc",
      y: "no_similarity_checks",
      stroke: "config_name",
      tip: true
    })
  ],
  y: {type: "log"},
  x: {type: "log"}
})
)}

function _12(md){return(
md`# Comparing Baseline vs Optimized`
)}

function _comp_data(ss,d3,agg_data)
{
  const calculateStats = (values) => {
    if (!values.length) return { sum: 0, q1: 0, q3: 0, median: 0 };

    const median = ss.median(values);

    return {
      sum: ss.sum(values),
      q1: ss.quantile(values, 0.25),
      q3: ss.quantile(values, 0.75),
      // iqr: ss.iqr(values),
      median: median,
    };
  };

  // Accepts either a string (property name) or a function (lambda)
  const extractAttributeValues = (data, attribute) => {
    if (typeof attribute === "function") {
      return data.map(attribute).filter((val) => val != null);
    } else if (
      typeof attribute === "object" &&
      attribute !== null &&
      typeof attribute.lambda === "function"
    ) {
      return data.map(attribute.lambda).filter((val) => val != null);
    } else {
      return data.map((d) => d[attribute]).filter((val) => val != null);
    }
  };

  const mannWhitneyU = (values1, values2) => {
    if (!values1.length || !values2.length)
      return { uStat: 0, pValue: 1, zScore: 0 };

    const n1 = values1.length;
    const n2 = values2.length;

    // Combine and rank all values
    const combined = [
      ...values1.map((v) => ({ value: v, group: 1 })),
      ...values2.map((v) => ({ value: v, group: 2 })),
    ];
    combined.sort((a, b) => a.value - b.value);

    // Assign ranks (handling ties by averaging)
    let currentRank = 1;
    for (let i = 0; i < combined.length; i++) {
      let tieCount = 1;
      while (
        i + tieCount < combined.length &&
        combined[i].value === combined[i + tieCount].value
      ) {
        tieCount++;
      }

      const avgRank = currentRank + (tieCount - 1) / 2;
      for (let j = 0; j < tieCount; j++) {
        combined[i + j].rank = avgRank;
      }

      i += tieCount - 1;
      currentRank += tieCount;
    }

    // Calculate U statistic
    const r1 = combined
      .filter((item) => item.group === 1)
      .reduce((sum, item) => sum + item.rank, 0);
    const u1 = r1 - (n1 * (n1 + 1)) / 2;
    const u2 = n1 * n2 - u1;
    const uStat = Math.min(u1, u2);

    // Normal approximation for p-value
    const meanU = (n1 * n2) / 2;
    const stdU = Math.sqrt((n1 * n2 * (n1 + n2 + 1)) / 12);
    const zScore = stdU > 0 ? (uStat - meanU) / stdU : 0;
    const pValue =
      2 * (1 - ss.cumulativeStdNormalProbability(Math.abs(zScore)));

    return {
      uStat: uStat,
      pValue: Math.max(0, Math.min(1, pValue)),
      zScore: zScore,
    };
  };
  const compareStats = (stats1, stats2, values1, values2, label1, label2) => {
    const mann_whitney_u_result = mannWhitneyU(values1, values2);

    return {
      sum_diff: stats1.sum - stats2.sum,
      sum_ratio: stats2.sum !== 0 ? stats1.sum / stats2.sum : null,
      sum_pct_reduction:
        stats2.sum !== 0
          ? ((stats1.sum - stats2.sum) / stats1.sum) * 100
          : null,
      median_diff: stats1.median - stats2.median,
      median_ratio: stats2.median !== 0 ? stats1.median / stats2.median : null,
      median_pct_reduction:
        stats2.median !== 0
          ? ((stats1.median - stats2.median) / stats1.median) * 100
          : null,
      q1_diff: stats1.q1 - stats2.q1,
      q1_ratio: stats2.q1 !== 0 ? stats1.q1 / stats2.q1 : null,
      q1_pct_reduction:
        stats2.q1 !== 0 ? ((stats1.q1 - stats2.q1) / stats1.q1) * 100 : null,
      q3_diff: stats1.q3 - stats2.q3,
      q3_ratio: stats2.q3 !== 0 ? stats1.q3 / stats2.q3 : null,
      q3_pct_reduction:
        stats2.q3 !== 0 ? ((stats1.q3 - stats2.q3) / stats1.q3) * 100 : null,
      mann_whitney_u_result,
      significant_005: mann_whitney_u_result.pValue < 0.05,
      significant_001: mann_whitney_u_result.pValue < 0.01,
    };
  };

  // attributes: array of strings or objects {lambda, name}
  const calculateAlgorithmStats = (data, attributes) => {
    const stats = {};

    attributes.forEach((attr) => {
      let key, values;
      if (typeof attr === "string") {
        key = attr;
        values = extractAttributeValues(data, attr);
      } else if (
        typeof attr === "object" &&
        attr !== null &&
        typeof attr.lambda === "function"
      ) {
        key = attr.name || "[lambda]";
        values = extractAttributeValues(data, attr);
      } else if (typeof attr === "function") {
        key = attr.name || "[lambda]";
        values = extractAttributeValues(data, attr);
      } else {
        return;
      }
      stats[key] = calculateStats(values);
    });

    return stats;
  };

  // Helper to sum an attribute for a dataset
  const sumAttribute = (data, attribute) => {
    const values = extractAttributeValues(data, attribute);
    return d3.sum(values);
  };

  const comparisons = [
    // Shallow Statement
    ["Baseline with Shallow Statement", "Optimized with Shallow Statement"],
    ["Baseline with Shallow Statement", "Optimized with Shallow Statement and Ngram Cache"],
    ["Baseline with Shallow Statement", "Optimized with Shallow Statement and Label Cache"],

    // Deep Statement
    ["Baseline with Deep Statement", "Optimized with Deep Statement"],
    [
      "Baseline with Deep Statement",
      "Optimized with Deep Statement and Ngram Cache",
    ],
    [
      "Baseline with Deep Statement",
      "Optimized with Deep Statement and Label Cache",
    ],

    // Opt Shallow vs Deep
    // [
    //   "Optimized with Statement",
    //   "Optimized with Deep Statement",
    // ],
    // [
    //   "Optimized with Statement and Ngram Cache",
    //   "Optimized with Deep Statement and Ngram Cache",
    // ],

    // Opt Cache comparison
    // [
    //   "Optimized with Deep Statement and Label Cache",
    //   "Optimized with Deep Statement and Ngram Cache",
    // ],
    // [
    //   "Optimized with Deep Statement",
    //   "Optimized with Deep Statement and Label Cache",
    // ],
    // [
    //   "Optimized with Deep Statement",
    //   "Optimized with Deep Statement and Ngram Cache",
    // ],
  ];

  // Define which attributes to calculate stats for
  // Now accepts strings or objects {lambda, name}
  const statsAttributes = [
    { lambda: (d) => d.duration_mean, name: "duration" },
    {
      lambda: (d) => d.duration_wo_similarity_mean,
      name: "duration_wo_similarity",
    },
    {
      lambda: (d) => d.diff_summary.leaves.similarity_checks,
      name: "number_similarity_checks",
    },
    {
      lambda: (d) => d.diff_summary.leaves.similarity_time,
      name: "leaves_similarity_time",
    },
    {
      lambda: (d) => d.diff_summary.bottomup.similarity_time,
      name: "bottomup_similarity_time",
    },
    {
      lambda: (d) => d.diff_summary.leaves.similarity_time / d.duration_mean,
      name: "leaves_similarity_ratio",
    },
    {
      lambda: (d) => d.diff_summary.bottomup.similarity_time / d.duration_mean,
      name: "bottomup_similarity_ratio",
    },
    {
      lambda: (d) => (d.duration_mean - (d.diff_summary.leaves.total_time + d.diff_summary.bottomup.total_time)) / d.duration_mean,
      name: "overhead_ratio",
    },

    // { lambda: d => d.diff_summary.leaves.characters_compared, name: "number_characters_compared" }
  ];

  const filtered_data = comparisons.map((names) => {
    const data = agg_data.filter((d) => names.includes(d.config_name));

    // Separate data by algorithm
    const algo1_data = data.filter((d) => d.config_name === names[0]);
    const algo2_data = data.filter((d) => d.config_name === names[1]);

    if (!algo1_data.length || !algo2_data.length)
      return {
        data: [],
        names: [],
        speedup_data: [],
        mean: 0,
        q1: 0,
        median: 0,
        q3: 0,
        slope: 0,
        intercept: 0,
        rSquared: 0,
        algo1_stats: {},
        algo2_stats: {},
        algo_comparison: {},
        speedup_stats: {},
        regression_stats: {},
      };

    const speedup_data = d3
      .groups(data, (d) => d.file_name)
      .map(([k, group]) => {
        if (group.length != 2) return;
        const [base, opt] = group[0].config_name.includes("Baseline")
          ? [group[0], group[1]]
          : [group[1], group[0]];
        let speedup = base.duration_mean / opt.duration_mean;
        if (speedup < 1) speedup = -1 / speedup;

        return {
          ...opt,
          speedup,
        };
      })
      .filter(Boolean);

    const similarity_time_ratio = d3
      .groups(data, (d) => d.file_name)
      .map(([k, group]) => {
        if (group.length != 2) return;
        const [base, opt] = group[0].config_name.includes("Baseline")
          ? [group[0], group[1]]
          : [group[1], group[0]];
        let base_ratio =
          base.diff_summary.leaves.similarity_time / base.duration_mean;
        let opt_ratio =
          opt.diff_summary.leaves.similarity_time / opt.duration_mean;

        return {
          base_ratio,
          opt_ratio,
        };
      })
      .filter(Boolean);

    const similarity_checks_reduction_data = d3
      .groups(data, (d) => d.file_name)
      .map(([k, group]) => {
        if (group.length != 2) return;
        const [base, opt] = group[0].config_name.includes("Baseline")
          ? [group[0], group[1]]
          : [group[1], group[0]];
        let ratio =
          base.diff_summary.leaves.similarity_checks /
          opt.diff_summary.leaves.similarity_checks;
        if (ratio < 1) ratio = -1 / ratio;

        return {
          ...opt,
          ratio,
        };
      })
      .filter(Boolean);

    const regression_stats = (() => {
      const regression = ss.linearRegression(
        speedup_data.map((d) => [d.loc, d.speedup]),
      );
      const line = ss.linearRegressionLine(regression);
      const ssTot = d3.sum(speedup_data, (d) =>
        Math.pow(d.speedup - d3.mean(speedup_data, (sd) => sd.speedup), 2),
      );
      const ssRes = d3.sum(speedup_data, (d) =>
        Math.pow(d.speedup - line(d.loc), 2),
      );
      const rSquared = 1 - ssRes / ssTot;

      return {
        slope: regression.m,
        intercept: regression.b,
        ssTot,
        ssRes,
        rSquared,
      };
    })();

    // Calculate stats for each algorithm separately
    const algo1_stats = calculateAlgorithmStats(algo1_data, statsAttributes);
    const algo2_stats = calculateAlgorithmStats(algo2_data, statsAttributes);
    const speedup_stats = {
      ...calculateStats(speedup_data.map((e) => e.speedup)),
      algo1_better: speedup_data.filter((d) => d.speedup < 0).length,
      algo2_better: speedup_data.filter((d) => d.speedup > 0).length,
      algo2_better_ratio:
        speedup_data.filter((d) => d.speedup > 0).length / speedup_data.length,
    };
    const similarity_time_ratio_algo1_stats = calculateStats(
      similarity_time_ratio.map((e) => e.base_ratio),
    );
    const similarity_time_ratio_algo2_stats = calculateStats(
      similarity_time_ratio.map((e) => e.opt_ratio),
    );
    const similarity_checks_reduction_stats = calculateStats(
      similarity_checks_reduction_data.map((e) => e.ratio),
    );

    // Compare the two algorithms
    const algo_comparison = {};
    statsAttributes.forEach((attr) => {
      let key;
      if (typeof attr === "string") {
        key = attr;
      } else if (
        typeof attr === "object" &&
        attr !== null &&
        typeof attr.lambda === "function"
      ) {
        key = attr.name || "[lambda]";
      } else if (typeof attr === "function") {
        key = attr.name || "[lambda]";
      } else {
        return;
      }
      if (algo1_stats[key] && algo2_stats[key]) {
        const values1 = extractAttributeValues(algo1_data, attr);
        const values2 = extractAttributeValues(algo2_data, attr);

        algo_comparison[key] = compareStats(
          algo1_stats[key],
          algo2_stats[key],
          values1,
          values2,
          names[0],
          names[1],
        );
      }
    });

    return {
      data,
      names,
      speedup_data,
      algo1_stats,
      algo2_stats,
      algo_comparison,
      speedup_stats,
      regression_stats,
      similarity_checks_reduction_ratio: similarity_checks_reduction_stats,
      total_similarity_ratios: {
        algo1: {
          leaves:
            algo1_stats.leaves_similarity_time.sum / algo1_stats.duration.sum,
          bottomup:
            algo1_stats.bottomup_similarity_time.sum / algo1_stats.duration.sum,
          total:
            (algo1_stats.leaves_similarity_time.sum +
              algo1_stats.bottomup_similarity_time.sum) /
            algo1_stats.duration.sum,
        },
        algo2: {
          leaves:
            algo2_stats.leaves_similarity_time.sum / algo2_stats.duration.sum,
          bottomup:
            algo2_stats.bottomup_similarity_time.sum / algo2_stats.duration.sum,
          total:
            (algo2_stats.leaves_similarity_time.sum +
              algo2_stats.bottomup_similarity_time.sum) /
            algo2_stats.duration.sum,
        },
      },
      total_overhead: {
        algo1: (algo1_stats.duration.sum - ss.sum(algo1_data.map(e => e.diff_summary.leaves.total_time + e.diff_summary.bottomup.total_time))) / algo1_stats.duration.sum,
        algo2: (algo2_stats.duration.sum - ss.sum(algo2_data.map(e => e.diff_summary.leaves.total_time + e.diff_summary.bottomup.total_time))) / algo2_stats.duration.sum,
      },
    };
  });

  return filtered_data;
}


function _14(comp_data,Plot)
{
   const data = comp_data.filter(e => e.names[0] == "Baseline with Deep Statement").map((e, i) => ({ratio: e.algo_comparison.duration.sum_ratio, cache: e.names[1].includes("Ngram") ? "With Ngram Cache": e.names[1].includes("Cache")? "With Label Cache": "No Cache"}))
  return Plot.plot({
    marks:[
      Plot.barY(data, {x: "cache", y: "ratio"}),
      Plot.text(data, {x: "cache", y: "ratio", text: e => e.ratio.toFixed(2) + "x", dy: 10, lineAnchor: "top", fill: "white"}),
  
    ],
    x: {label: "Variants of HyperAST-adapted algorithm", labelOffset: 50, },
    y: {label: "Speedup ratio compared to Baseline with Deep Statement",  grid: true, },
    marginTop: 50,
    marginBottom: 70,
    style: {fontSize: 16}
  })
}


function _15(comp_data,Plot)
{
   const data = comp_data.filter(e => e.names[0] == "Baseline with Statement").flatMap((e, i) => e.speedup_data.map(s => (
     {
       ratio: s.speedup, 
       cache: e.names[1].includes("Ngram") ? "With Ngram Cache": e.names[1].includes("Cache")? "With Label Cache": "No Cache"
     })))
  
  return Plot.plot({
    marks:[
      Plot.boxY(data, {x: "cache", y: "ratio"}),
    ],
    x: {label: "Versions of HyperAST-adapted algorithm", labelOffset: 50},
    y: {label: "Speedup ratio compared to Baseline with Deep Statement",  grid: true},
    marginTop: 50,
    marginBottom: 70,
    style: {fontSize: 13}
  })
}


function _pivot_table(comp_data)
{
  function getLeafPaths(obj, prefix = "") {
  let paths = [];
  for (const key in obj) {
    if (key === "names") continue; // skip names
    const value = obj[key];
    const path = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      paths = paths.concat(getLeafPaths(value, path));
    } else if (typeof value !== "object" || value === null) {
      paths.push(path);
    }
  }
  return paths;
}

function getValueByPath(obj, path) {
  return path.split('.').reduce((acc, key) => acc && acc[key], obj);
}

function extractAllStats(comp_data) {
  // Get all unique leaf stat paths from all entries
  const allPaths = new Set();
  comp_data.forEach(entry => {
    Object.keys(entry).forEach(key => {
      if (key !== "names") {
        getLeafPaths({ [key]: entry[key] }, "").forEach(p => allPaths.add(p));
      }
    });
  });

  // Build the column names from the names array in each comp_data entry
  const columnNames = comp_data.map(
    entry => `${entry.names[0]} vs ${entry.names[1]}`
  );

  // For each stat path, build a row object
  const rows = Array.from(allPaths).sort().map(stat_name => {
    const row = { stat_name };
    comp_data.forEach((entry, idx) => {
      row[columnNames[idx]] = getValueByPath(entry, stat_name);
    });
    return row;
  });

  return rows;
}
  return extractAllStats(comp_data)
}


function _17(comp_data){return(
JSON.stringify(comp_data.map(e => ({...e, data:undefined, speedup_data: undefined})), null, 2)
)}

function _stats_title(comp_data){return(
function stats_title(idx) {
  return `${comp_data[idx].names?.[0]} vs. ${comp_data[idx].names?.[1]}`
}
)}

function _stats_table(md,comp_data){return(
function stats_table(idx) {
  return md`
    | Statistic | ${comp_data[idx].names?.[0]} vs. ${comp_data[idx].names?.[1]} |
    |-----------|---------------------------|
    | Mean      | ${comp_data[idx].speedup_stats.mean?.toFixed(2)} | 
    | Median    | ${comp_data[idx].speedup_stats.median?.toFixed(2)} | 
    | Q1        | ${comp_data[idx].speedup_stats.q1?.toFixed(2)} | 
    | Q3        | ${comp_data[idx].speedup_stats.q3?.toFixed(2)} | 
    | Slope     | ${comp_data[idx].regression_stats.slope?.toFixed(2)} | 
    | Intercept | ${comp_data[idx].regression_stats.intercept?.toFixed(2)} | 
    | R²        | ${comp_data[idx].regression_stats.rSquared?.toFixed(2)} | 
    `
}
)}

function _speedup_plot(Plot,comp_data){return(
function speedup_plot(idx) {
  return Plot.plot({
      marks: [
        Plot.dot(comp_data[idx].speedup_data, {x: "loc", y: "speedup", stroke: e => e.speedup > 0}),
        Plot.linearRegressionY(comp_data[idx].speedup_data, {
          x: "loc",
          y: "speedup",
          
        }),
        Plot.ruleY([0])
      ],
    x: {label: "Lines of Code in Buggy File"},
    y: {label: "Speedup of Optimized over Baseline"},
    style: {fontSize: 16},
    marginBottom: 40,
    marginTop: 30,
    marginLeft: 50
    }
    
                   
  );
  }
)}

function _data_plot(Plot,comp_data){return(
function data_plot(idx) {
  return Plot.plot({
      marks: [
        Plot.dot(comp_data[idx].data, {x: "loc", y: e => e.duration_mean / 1000, stroke: "config_name", tip: true}),
      ],
    // margin: 50,
      x: {label: "Lines of Code in Buggy File"},
    y: {label: "Runtime duration in seconds"},
    marginTop: 30,
    marginBottom: 50,
    color: {legend: true},
    style: {fontSize: 16}
    }
  );
  }
)}

function _22(speedup_plot){return(
speedup_plot(3)
)}

function _23(data_plot){return(
data_plot(3)
)}

function _24(htl){return(
htl.html`<style>
  .large-font * {
    font-size: 16px;
    margin-bottom: 20px;
  }
</style>`
)}

function _25(Plot,comp_data){return(
Plot.plot({
      marks: [
        Plot.dot([...comp_data[0].data, ...comp_data[3].data], 
                 {
                   x: "loc", 
                   y: e => e.duration_mean / 1000, 
                   // y: "duration_wo_similarity_mean",
                   stroke: "config_name", tip: true, symbol: "config_name", r: 4}),
      ],
    // margin: 50,
      x: {label: "Lines of Code in Buggy File", type: "log"},
    y: {label: "Runtime duration in seconds", type: "log"},
    marginTop: 30,
  marginLeft: 60,
    marginBottom: 50,
    symbol: {legend: true, className: 'large-font'},
    style: {fontSize: 16}
    }
  )
)}

function _26(htl){return(
htl.html`<style>
  svg p, span { 
    font-size: 16px !important 
  } 
</style>`
)}

function _27(comp_data,Plot)
{
  const data = [0, 1, 2, 3, 4, 5].map(i => comp_data[i].speedup_data.map(d => ({...d, short_name: d.config_name.replace("with Statement", "with Shallow Statement").replace("Optimized with", "").replace("Statement", "")})))
  const data_flat = data.flatMap(e => e)
  return Plot.plot({
    marks: [
      Plot.dot(
        data_flat, {
          x: "loc",
          y: "speedup",
          opacity: 0.7,
          // r: 6,
          tip: true,
          stroke: "short_name",
          symbol: "short_name"
        }
      ),
      ...data.map(data =>
        Plot.linearRegressionY(data, {
          x: "loc",
          y: "speedup",
          stroke: "short_name"
        }),
      ),
      Plot.ruleY([0])
    ],
    x: {
      label: "Lines of Code in Buggy File",
    },
    y: {
      label: "Speedup of Optimized over Baseline"
    },
    symbol: {legend: true},
    style: {
      fontSize: 16
    },
    marginBottom: 40,
    marginTop: 30,
    marginLeft: 60,
    
  })
}


function _28(comp_data,ss){return(
comp_data.map(e => ss.min(e.speedup_data.map(e => e.speedup)))
)}

function _29(comp_data,Plot)
{
  const data = [0, 1, 2, 3, 4, 5].map(i => comp_data[i].data.map(d => ({...d, short_name: d.config_name.replace("with Statement", "with Shallow Statement").replace("Optimized with", "").replace("Statement", "")})))
  const data_flat = data.flatMap(e => e)
  return Plot.plot({
    marks: [
      Plot.dot(
        data_flat, {
          x: "loc",
          y: "speedup",
          opacity: 0.7,
          // r: 6,
          tip: true,
          stroke: "short_name",
          symbol: "short_name"
        }
      ),
      ...data.map(data =>
        Plot.linearRegressionY(data, {
          x: "loc",
          y: "speedup",
          stroke: "short_name"
        }),
      ),
      Plot.ruleY([0])
    ],
    x: {
      label: "Lines of Code in Buggy File",
    },
    y: {
      label: "Speedup of Optimized over Baseline"
    },
    symbol: {legend: true},
    style: {
      fontSize: 16
    },
    marginBottom: 40,
    marginTop: 30,
    marginLeft: 60,
    
  })
}


function _30(stats_title,stats_table,speedup_plot,data_plot,md){return(
md`## ${stats_title(0)}
${stats_table(0)}
${speedup_plot(0)}
${data_plot(0)}

## ${stats_title(1)}
${stats_table(1)}
${speedup_plot(1)}
${data_plot(1)}

## ${stats_title(2)}
${stats_table(2)}
${speedup_plot(2)}
${data_plot(2)}

## ${stats_title(3)}
${stats_table(3)}
${speedup_plot(3)}
${data_plot(3)}

## ${stats_title(4)}
${stats_table(4)}
${speedup_plot(4)}
${data_plot(4)}

## ${stats_title(5)}
${stats_table(5)}
${speedup_plot(5)}
${data_plot(5)}

## ${stats_title(6)}
${stats_table(6)}
${speedup_plot(6)}
${data_plot(6)}`
)}

function _31(md){return(
md`# Single comparison`
)}

function _comp_data1(agg_data){return(
agg_data.filter(e => e.config_name == "Optimized with Deep Statement and Ngram Caching" || e.config_name == "Baseline with Deep Statement")
)}

function _speedup_data(d3,comp_data){return(
d3.groups(comp_data, d => d.file_name).map(([k, group]) => {
  if(group.length != 2) return;
  const [base, opt] = group[0].config_name.includes("Baseline") ? [group[0], group[1]] : [group[1], group[0]]
  let speedup = base.duration_mean / opt.duration_mean
  if (speedup < 1) 
    speedup = -1 / speedup 

  return {
    ...opt,
    speedup
  }
  
}).filter(Boolean)
)}

function _34(Plot,speedup_data){return(
Plot.plot({
    marks: [
      Plot.dot(speedup_data, {x: "loc", y: "speedup", color: e => e.speedup > 0}),
      Plot.linearRegressionY(speedup_data, {
        x: "loc",
        y: "speedup",
        
      }),
      Plot.ruleY([0])
    ]
  }
)
)}

function _35(Plot,speedup_data){return(
Plot.plot({
  marks: [
    Plot.boxX(speedup_data.map(e => e.speedup))
  ],
  y: {label: "Speedup of Optimized over Baseline"}
})
)}

function _stats(speedup_data,ss,d3)
{
  const sus = speedup_data.map(e => e.speedup)
  const mean = ss.mean(sus)
  const q1 = ss.quantile(sus, 0.25)
  const median = ss.median(sus)
  const q3 = ss.quantile(sus, 0.75)

  const regression = ss.linearRegression(speedup_data.map(d => [d.loc, d.speedup]));
  const line = ss.linearRegressionLine(regression); 
  const ssTot = d3.sum(speedup_data, d => Math.pow(d.speedup - d3.mean(speedup_data, sd => sd.speedup), 2));
  const ssRes = d3.sum(speedup_data, d => Math.pow(d.speedup - line(d.nodes), 2));
  const rSquared = 1 - (ssRes / ssTot);

  return {
    mean,
    q1,
    median,
    q3,
    slope: regression.m,
    intercept: regression.b,
    rSquared: rSquared,
  };
}


function _37(Plot,comp_data){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.dot(comp_data, {
      x: "loc",
      y: "no_similarity_checks",
      stroke: "config_name",
      tip: true
    })
  ],
  y: {type: "log"}
})
)}

function _38(Plot,data_opt){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.lineY(data_opt, {
      x: "node_count",
      y: "duration_mean",
      stroke: "config_name",
      tip: true
    })
  ],
  y: {type: "log"}
})
)}

function _39(Plot,agg_data){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.lineY(agg_data.map(e => ({...e, time_wo_sim: e.duration_mean - e.l_sim_ms_mean})), {
      x: "node_count",
      y: "time_wo_sim",
      stroke: "config_name",
      tip: true
    })
  ],
  y: {type: "log"}
})
)}

function _40(Plot,agg_data){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.dot(agg_data, {
      filter: e => e.config_name == "Baseline Statement" || e.config_name == "Optimized with Statement and Label Cache",
      x: "node_count",
      y: "duration_mean",
      stroke: "config_name",
      tip: true
    }),
    Plot.linearRegressionY(agg_data, {
      filter: e => e.config_name == "Baseline Statement",
      x: "node_count",
      y: "duration_mean",
      stroke: "config_name",
    }),
    Plot.linearRegressionY(agg_data, {
      filter: e => e.config_name == "Optimized with Statement and Label Cache",
      x: "node_count",
      y: "duration_mean",
      stroke: "config_name",
    })
  ],
  y: {type: "log"}
})
)}

function _41(Plot,agg_data){return(
Plot.plot({
  color: { legend: true },
  marks: [
    Plot.areaY(agg_data, {
      x: "node_count",
      y1: "duration_mean_lower",
      y2: "duration_mean_upper",
      fill: "config_name",
      opacity: 0.15,
      
    }),
    Plot.line(agg_data, {x: "node_count", y: "duration_mean", stroke: "config_name", tip: true}),

    Plot.dot(agg_data, {
      x: "node_count",
      y: "duration_mean",
      stroke: "config_name",
      fill: "white"
    })
  ],
    marginLeft: 50,
  x: {label: "Nodes"},  
  y: {label: "Mean Runtime in ms", grid: true, },
  color: {legend: true, label: "Algorithm"},
  style: {fontSize: "12px"}
})
)}

export default function define(runtime, observer) {
  const main = runtime.module();
  function toString() { return this.url; }
  const fileAttachments = new Map([
    ["cd_benchmark_20250619_062955_r1_tc1046_cfg8@2.jsonl", {url: new URL("./files/fcf1b66bfb3e542a39a2172cf346b368b6c68be19e861ae01d83df7876d90bbfcd4a7e276860ac0eee9c8165b498a7f5ce66bfc8b35e90188640abbe925ef644.bin", import.meta.url), mimeType: "application/octet-stream", toString}]
  ]);
  main.builtin("FileAttachment", runtime.fileAttachments(name => fileAttachments.get(name)));
  main.variable(observer()).define(["md"], _1);
  main.variable(observer("ss")).define("ss", ["require"], _ss);
  main.variable(observer("jStat")).define("jStat", ["require"], _jStat);
  main.variable(observer("raw_data")).define("raw_data", ["FileAttachment"], _raw_data);
  main.variable(observer("groupedData")).define("groupedData", ["d3","raw_data"], _groupedData);
  main.variable(observer("agg_data")).define("agg_data", ["d3","raw_data","ss"], _agg_data);
  main.variable(observer()).define(["ss","agg_data"], _7);
  main.variable(observer()).define(["__query","agg_data","invalidation"], _8);
  main.variable(observer("data_opt")).define("data_opt", ["agg_data"], _data_opt);
  main.variable(observer("data_base")).define("data_base", ["agg_data"], _data_base);
  main.variable(observer()).define(["Plot","agg_data"], _11);
  main.variable(observer()).define(["md"], _12);
  main.variable(observer("comp_data")).define("comp_data", ["ss","d3","agg_data"], _comp_data);
  main.variable(observer()).define(["comp_data","Plot"], _14);
  main.variable(observer()).define(["comp_data","Plot"], _15);
  main.variable(observer("pivot_table")).define("pivot_table", ["comp_data"], _pivot_table);
  main.variable(observer()).define(["comp_data"], _17);
  main.variable(observer("stats_title")).define("stats_title", ["comp_data"], _stats_title);
  main.variable(observer("stats_table")).define("stats_table", ["md","comp_data"], _stats_table);
  main.variable(observer("speedup_plot")).define("speedup_plot", ["Plot","comp_data"], _speedup_plot);
  main.variable(observer("data_plot")).define("data_plot", ["Plot","comp_data"], _data_plot);
  main.variable(observer()).define(["speedup_plot"], _22);
  main.variable(observer()).define(["data_plot"], _23);
  main.variable(observer()).define(["htl"], _24);
  main.variable(observer()).define(["Plot","comp_data"], _25);
  main.variable(observer()).define(["htl"], _26);
  main.variable(observer()).define(["comp_data","Plot"], _27);
  main.variable(observer()).define(["comp_data","ss"], _28);
  main.variable(observer()).define(["comp_data","Plot"], _29);
  main.variable(observer()).define(["stats_title","stats_table","speedup_plot","data_plot","md"], _30);
  main.variable(observer()).define(["md"], _31);
  main.variable(observer("comp_data1")).define("comp_data1", ["agg_data"], _comp_data1);
  main.variable(observer("speedup_data")).define("speedup_data", ["d3","comp_data"], _speedup_data);
  main.variable(observer()).define(["Plot","speedup_data"], _34);
  main.variable(observer()).define(["Plot","speedup_data"], _35);
  main.variable(observer("stats")).define("stats", ["speedup_data","ss","d3"], _stats);
  main.variable(observer()).define(["Plot","comp_data"], _37);
  main.variable(observer()).define(["Plot","data_opt"], _38);
  main.variable(observer()).define(["Plot","agg_data"], _39);
  main.variable(observer()).define(["Plot","agg_data"], _40);
  main.variable(observer()).define(["Plot","agg_data"], _41);
  return main;
}
