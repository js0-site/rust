import redis from "./R.js";
import utf8e from "@3-/utf8";

const DOMAIN_HOST = utf8e("smtpDomainHost:"),
  HOST_DKIM = utf8e("smtpHostDkim:"),
  HOST_DKIM_KEY = utf8e("smtpHostDkimKey:"),
  HOST_DKIM_PUB = utf8e("smtpHostDkimPub:"),
  HOST_ID = "smtpHostId",
  DKIM_SK = "smtpDkimSk",
  concat = (...arrays) => {
    const res = new Uint8Array(arrays.reduce((acc, cur) => acc + cur.length, 0));
    let pos = 0;
    for (const arr of arrays) {
      res.set(arr, pos);
      pos += arr.length;
    }
    return res;
  },
  write = (val) => {
    if (!val) {
      return new Uint8Array(0);
    }
    const buf = new Uint8Array(8);
    new DataView(buf.buffer).setBigUint64(0, BigInt(val), true);
    let len = 8;
    while (len > 0 && buf[len - 1] === 0) {
      --len;
    }
    return buf.subarray(0, len);
  };

export default async (domain) => {
  const domain_key = concat(DOMAIN_HOST, utf8e(domain));
  let [dkim_sk_bytes, host_id_bytes] = await Promise.all([
    redis.getBuffer(DKIM_SK),
    redis.getBuffer(Buffer.from(domain_key)),
  ]);

  const tasks = [];
  if (!dkim_sk_bytes) {
    dkim_sk_bytes = crypto.getRandomValues(new Uint8Array(32));
    tasks.push(redis.set(DKIM_SK, Buffer.from(dkim_sk_bytes)));
    console.log("未检测到全局 DKIM 密钥，已自动生成并保存。");
  }

  if (!host_id_bytes) {
    const host_id = await redis.incr(HOST_ID);
    host_id_bytes = write(host_id);
    tasks.push(redis.set(Buffer.from(domain_key), Buffer.from(host_id_bytes)));
  }

  if (tasks.length > 0) {
    await Promise.all(tasks);
  }

  const host_dkim_key = concat(HOST_DKIM, host_id_bytes),
    host_dkim_private_key = concat(HOST_DKIM_KEY, host_id_bytes),
    host_dkim_public_key = concat(HOST_DKIM_PUB, host_id_bytes);

  let [selector, private_key_der, public_key_der] = await Promise.all([
    redis.get(Buffer.from(host_dkim_key)),
    redis.getBuffer(Buffer.from(host_dkim_private_key)),
    redis.getBuffer(Buffer.from(host_dkim_public_key)),
  ]);

  const write_tasks = [];
  if (!selector) {
    selector = "rsa";
    write_tasks.push(redis.set(Buffer.from(host_dkim_key), selector));
  }

  if (!private_key_der || !public_key_der) {
    const key_pair = await crypto.subtle.generateKey(
      {
        name: "RSASSA-PKCS1-v1_5",
        modulusLength: 2048,
        publicExponent: new Uint8Array([1, 0, 1]),
        hash: "SHA-256",
      },
      true,
      ["sign", "verify"],
    );

    private_key_der = new Uint8Array(await crypto.subtle.exportKey("pkcs8", key_pair.privateKey));
    public_key_der = new Uint8Array(await crypto.subtle.exportKey("spki", key_pair.publicKey));

    write_tasks.push(
      redis.set(Buffer.from(host_dkim_private_key), Buffer.from(private_key_der)),
      redis.set(Buffer.from(host_dkim_public_key), Buffer.from(public_key_der)),
    );
  }

  if (write_tasks.length > 0) {
    await Promise.all(write_tasks);
  }

  const pk_base64 = Buffer.from(public_key_der).toString("base64"),
    txt_record = "v=DKIM1; k=rsa; p=" + pk_base64;

  return [selector, txt_record];
};
