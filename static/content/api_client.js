import ky from "https://esm.sh/ky";
import { token } from "./api.js";

export let get_info = async () => await ky.get("/na/info").json();

export const user = {
    info: async (username) =>
        await ky
            .get(`/api/user/${username}`, {
                headers: { Authorization: token() },
            })
            .json(),
    login: async (username, password) =>
        await ky
            .post("/na/user", {
                json: { username, password, identifier: "password" },
                throwHttpErrors: false,
            })
            .json(),
    register: async (username, password) =>
        await ky
            .put("/na/user", {
                json: { username, password, identifier: "password" },
                throwHttpErrors: false,
            })
            .json(),
};

export const library = {
    all: async () =>
        window.session.get("lib-all") ||
        window.session.set(
            "lib-all",
            await ky
                .get(`/api/library/all`, {
                    headers: { Authorization: token() },
                })
                .json()
        ),
    get: async (id) =>
        window.session.get(`lib-${id}`) ||
        window.session.set(
            `lib-${id}`,
            await ky
                .get(`/api/library/${id}`, {
                    headers: { Authorization: token() },
                })
                .json()
        ),
    scan: async (id) =>
        await ky
            .post(`/api/library/${id}/scan`, {
                headers: { Authorization: token() },
            })
            .json(),
    create: async (name, path, depth) => await ky.put(`/api/library`, {
	    headers: { Authorization: token() },
	    throwHttpErrors: false,
	    json: { name, path, depth }
    }).json(),
    delete: async (id) => await ky.delete(`/api/library/${id}`, { headers: { Authorization: token() },
		throwHttpErrors: false }).json(),

};

export const file = {
    get: async (id) =>
        window.session.get(`file-${id}`) ||
        window.session.set(
            `file-${id}`,
            await ky
                .get(`/api/file/${id}`, {
                    headers: { Authorization: token() },
                })
                .json()
        ),
};
