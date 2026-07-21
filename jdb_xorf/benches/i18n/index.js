import en from "./en.js";
import zh from "./zh.js";

export const RESOURCES = { en, zh };

const sysLang = process.env.LANG || process.env.LC_ALL || "";
export const CURRENT_LANG = sysLang.includes("zh") ? "zh" : "en";

export const LABELS = RESOURCES[CURRENT_LANG];
export const LANG = CURRENT_LANG;

export default LABELS;
