import ky from 'https://esm.sh/ky';
import { token } from "./api.js";

export let get_info = async () => await ky.get("/na/info").json();

export const user = {
    info: async (username) => await ky.get(`/api/user/${username}`, { headers: { Authorization: token() } }).json(),
    login: async (username, password) => await ky.post("/na/user", {json: { username, password, identifier: "password"}, throwHttpErrors:false}).json(),
    register: async (username, password) => await ky.put("/na/user", {json: { username, password, identifier: "password"}, throwHttpErrors:false}).json(),
}


export const library = {
    all: async () => await ky.get(`/api/library/all`, { headers: { Authorization: token() } }).json(),
    get: async (id) => await ky.get(`/api/library/${id}`, { headers: { Authorization: token() } }).json(),
    // create: async () => 
}