import { token } from "./api.js";

export const http = {
	_fetch: (path, method, options) => {
		let headers = options.noauth ? {} : { Authorization: token() };
		let fetch_options = { method: method };
		if (options.hasOwnProperty("json")) {
			fetch_options.body = JSON.stringify(options.json);
			headers["Content-Type"] = "application/json";
		}
		fetch_options.headers = { ...fetch_options.headers, ...headers };
		return new Feetch(fetch(path, fetch_options));
	},
	get(path, options = {}) { return this._fetch(path, "get", options); },
	post(path, options = {}) { return this._fetch(path, "post", options); },
	delete(path, options = {}) { return this._fetch(path, "delete", options); },
	put(path, options = {}) { return this._fetch(path, "put", options); },
}

class Feetch {
	promise = null;
	constructor(promise) {
		this.promise = promise;
	}

	async json() {
		return await (await this.promise).json();
	}
}

