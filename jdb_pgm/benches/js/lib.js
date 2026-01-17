export const ALGORITHM_COLORS = {
    jdb_pgm: "#10b981", // Emerald 500
    external_pgm: "#3b82f6", // Blue 500
    hashmap: "#f59e0b", // Amber 500
    binary_search: "#ef4444", // Red 500
    btreemap: "#8b5cf6", // Violet 500
};

export const ALGORITHM_NAMES = {
    binary_search: "Binary Search",
    btreemap: "BTreeMap",
    hashmap: "HashMap",
    jdb_pgm: "jdb_pgm",
    external_pgm: "pgm_index",
};

export const ALGORITHM_NAMES_ZH = {
    binary_search: "二分查找",
    btreemap: "BTreeMap",
    hashmap: "HashMap",
    jdb_pgm: "jdb_pgm",
    external_pgm: "pgm_index",
};

export const getColor = (algo) => ALGORITHM_COLORS[algo] || "#000000";

export const fmtTime = (ns) => {
    if (ns < 1000) return `${ns.toFixed(2)}ns`;
    if (ns < 1000000) return `${(ns / 1000).toFixed(2)}µs`;
    if (ns < 1000000000) return `${(ns / 1000000).toFixed(2)}ms`;
    return `${(ns / 1000000000).toFixed(2)}s`;
};

export const fmtThroughput = (mops) => `${(mops / 1000000).toFixed(2)}M/s`;

export const formatDataSize = (d) => d.toLocaleString();

export const formatMemory = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

export const groupByDataSize = (results) => {
    const grouped = {};
    for (const r of results) {
        if (!grouped[r.data_size]) grouped[r.data_size] = [];
        grouped[r.data_size].push(r);
    }
    return grouped;
};

export const getSortedDataSizes = (grouped) =>
    Object.entries(grouped).sort((a, b) => parseInt(a[0]) - parseInt(b[0]));
