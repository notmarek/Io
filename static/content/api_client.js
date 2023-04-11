import ky from 'https://esm.sh/ky';

export let get_info = async () => await ky.get("/na/info").json();