import { user } from "./api_client.js";
export let navigate = (p) => {
  history.pushState({}, document.title, p);
  window.dispatchEvent(new Event("popstate"));
};

export let self = {
  get_username: async () => {
    let u = localStorage.getItem("username");
    if (!u) {
      return (await save_user_info())["username"];
    }
    return u;
  },
  get_permissions: async () => {
    let u = localStorage.getItem("permissions");
    if (!u) {
      return (await save_user_info())["permissions"];
    }
    return u;
  },
};

export let save_user_info = async () => {
  let info;
  try {
    info = await user.info("@me");
    localStorage.setItem("username", info.username);
    localStorage.setItem("permissions", info.permissions);
  } catch {
    info = { permissions: "" };
  }
  return info;
};

export let token = () => {
  let token = localStorage.getItem("token");
  if (!token) {
    return null;
  }
  return `${localStorage.getItem("token_type")} ${token}`;
};

export const save_tokens_from_response = (res) => {
  localStorage.setItem("token", res.token);
  localStorage.setItem("refresh_token", res.refresh_token);
  localStorage.setItem("token_type", res.token_type);
  localStorage.setItem("token_exp", res.expiration);
  localStorage.setItem("file_token", res.file_token);
};
export let submit = {
  login: async (event) => {
    event.preventDefault();
    let form = event.target;
    let uname = form.querySelector("input[name='username']").value;
    let passwd = form.querySelector("input[name='password']").value;
    let res = await user.login(uname, passwd);
    if (res.status !== "error") {
      save_tokens_from_response(res);
      await save_user_info();
      navigate("/");
    } else {
      document.querySelector("#error").innerText = res.error;
    }
    return false;
  },
  register: async (event) => {
    event.preventDefault();
    let form = event.target;
    let uname = form.querySelector("input[name='username']").value;
    let passwd = form.querySelector("input[name='password']").value;
    let res = await user.register(uname, passwd);
    if (res.status !== "error") {
      save_tokens_from_response(res);
      await save_user_info();
      navigate("/");
    } else {
      document.querySelector("#error").innerText = res.error;
    }
    return false;
  },
};
