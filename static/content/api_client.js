import ky from 'https://esm.sh/ky';
import { token } from "./api.js";

export let get_info = async () => await ky.get("/na/info").json();

export let user = {
    info: async (username) => await ky.get(`/api/user/${username}`, { headers: { Authorization: token() } }).json(),
    login: async (username, password) => await ky.post("/na/user", {json: { username, password, identifier: "password"}, throwHttpErrors:false}).json(),
    register: async (username, password) => await ky.put("/na/user", {json: { username, password, identifier: "password"}, throwHttpErrors:false}).json(),
}
