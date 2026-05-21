import Redis from "@3-/ioredis";

const { R_NODE, R_PORT, R_PASSWORD, R_SENTINEL_PASSWORD, R_SENTINEL_NAME, R_SENTINEL_PORT, R_HOST } = process.env,
  default_sentinel_port = R_SENTINEL_PORT ? parseInt(R_SENTINEL_PORT, 10) : 26379,
  sentinels = R_NODE
    ? R_NODE.trim()
        .split(/\s+/)
        .map((i) => {
          const [host, port_str] = i.split(":");
          const port = port_str ? parseInt(port_str, 10) : default_sentinel_port;
          return { host, port };
        })
    : [],
  option = {
    dropBufferSupport: false,
  };

Object.assign(
  option,
  sentinels.length > 0
    ? {
        sentinels,
        name: R_SENTINEL_NAME,
        sentinelPassword: R_SENTINEL_PASSWORD,
        password: R_PASSWORD,
      }
    : {
        host: R_HOST,
        port: +R_PORT,
        password: R_PASSWORD,
      },
);

const redis = Redis(option),
  withR = async (func) => {
    try {
      await func(redis);
    } catch (error) {
      console.error("操作失败:", error);
      process.exit(1);
    } finally {
      await redis.quit();
    }
  };

export default redis;
export { withR };
