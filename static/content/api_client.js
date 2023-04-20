import ky from 'https://esm.sh/ky';

export let get_info = async () => await ky.get("/na/info").json();

export let login = async (username, password) => await ky.post("/na/user", {json: { username, password, identifier: "password"}, throwHttpErrors:false}).json();