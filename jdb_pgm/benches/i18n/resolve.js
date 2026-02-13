import { join } from "path";

const ENV = (process.env.LANG || process.env.LANGUAGE || "").toLowerCase(),
  LANG = ENV.includes("zh") ? "zh" : "en";

export default (dir, ext, lang = LANG) =>
  join(import.meta.dirname, dir, `${lang}.${ext}`);
