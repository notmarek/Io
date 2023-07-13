import { http } from "./http.js";
export let get_info = async () => window.session.get("info") ||
								  window.session.set("info",
									  await http.get("/na/info").json());

export const user = {
    info: async (username) =>
        await http
            .get(`/api/user/${username}`)
            .json(),
	all: async () => 
		await http
			.get("/api/users")
			.json(),
	edit: async (uid, data) => 
		await http
			.post(`/api/user/${uid}`, {
					json: data,
			})
			.json(),
    login: async (username, password) =>
        await http
            .post("/na/user", {
                json: { username, password, identifier: "password" },
		noauth: 1,
            })
            .json(),
	refresh_token: async (refresh_token) => 
		await http
			.post("/na/user", {
				json: { refresh_token, identifier: "refresh_token" },
				noauth: 1 
			})
			.json(),
    register: async (username, password) =>
        await http
            .put("/na/user", {
                json: { username, password, identifier: "password" },
		noauth: 1,
            })
            .json(),
};

export const library = {
    all: async () =>
        window.session.get("lib-all") ||
        window.session.set(
            "lib-all",
            await http
                .get(`/api/library/all`)
                .json()
        ),
    get: async (id) =>
        window.session.get(`lib-${id}`) ||
        window.session.set(
            `lib-${id}`,
            await http
                .get(`/api/library/${id}`)
                .json()
        ),
    scan: async (id) =>
        await http
            .post(`/api/library/${id}/scan`)
            .json(),
    create: async (name, path, depth) => await http.put(`/api/library`, {
	    json: { name, path, depth }
    }).json(),
    delete: async (id) => await http.delete(`/api/library/${id}`).json(),

};

export const file = {
    get: async (id) =>
        window.session.get(`file-${id}`) ||
        window.session.set(
            `file-${id}`,
            await http
                .get(`/api/file/${id}`)
                .json()
        ),
};
