import { token } from "./api.js";
const proxy_handler = {
  get(target, prop, _receiver) {
    return async (...args) => {
      const res = await target;
      return await res[prop].apply(res, args);
    };
  },
};
export const http = {
  _fetch: (path, method, options) => {
    let headers = options.noauth ? {} : { Authorization: token() };
    let fetch_options = { method: method };
    if (options.hasOwnProperty("json")) {
      fetch_options.body = JSON.stringify(options.json);
      headers["Content-Type"] = "application/json";
    }
    fetch_options.headers = { ...fetch_options.headers, ...headers };
    return new Proxy(fetch(path, fetch_options), proxy_handler);
  },
  get(path, options = {}) {
    return this._fetch(path, "get", options);
  },
  post(path, options = {}) {
    return this._fetch(path, "post", options);
  },
  delete(path, options = {}) {
    return this._fetch(path, "delete", options);
  },
  put(path, options = {}) {
    return this._fetch(path, "put", options);
  },
};
