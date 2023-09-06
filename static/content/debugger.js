export const dbgr = async (path, self, renderModule, log) => {
  let container = document.createElement("div");

  container.style =
    "z-index:99999999; position: absolute; top:0;left:0;background: #000000af;padding:5px;";
  container.innerHTML = `[Server] host: ${location.host}
    <br>[User] username: ${await self.get_username()}
    permissions: ${await self.get_permissions()}
    <br>[Modules] last: <span id="dbgr-path">${path}</span>
    rendered: <span id="dbgr-num-modules">null</span>
    <br>[Cache] objects: <span id="dbgr-cache-size">null</span>
    hits: <span id="dbgr-cache-hits">null</span>
    misses: <span id="dbgr-cache-misses">null</span>
    invalid: <span id="dbgr-cache-inv">0</span>`;
  container.hidden = 1;
  let cache_hits = 0;
  let cache_misses = 0;
  after_hook(window.session, "set", (k, v) => {
    log("session.set", `Set '${k}' to '${v}'`);
    document.getElementById("dbgr-cache-size").innerHTML =
      Object.keys(window.session).length - 4;
    document.getElementById("dbgr-cache-inv").innerHTML =
      window.session.invalid.length;
  });
  after_hook(window.session, "invalidate", (k) => {
    log("session.invalidate", `Invalidated ${k}`);
    document.getElementById("dbgr-cache-inv").innerHTML =
      window.session.invalid.length;
  });
  after_hook(window.session, "get", (k, res) => {
    log("session.get", `Cache ${res ? "hit" : "miss"} on key "${k}".`);
    res ? cache_hits++ : cache_misses++;
    document.getElementById("dbgr-cache-hits").innerHTML = cache_hits;
    document.getElementById("dbgr-cache-misses").innerHTML = cache_misses;
  });
  before_hook(
    localStorage,
    "setItem",
    (k, v) => log("localStorage.setItem", `Set '${k}' to '${v}'`),
    true,
  );
  after_hook(
    localStorage,
    "getItem",
    (k, res) => log("localStorage.getItem", `Get '${k}' - value: '${res}'`),
    true,
  );
  before_hook(
    localStorage,
    "clear",
    () => log("localStorage.clear", `Clear.`),
    true,
  );
  before_hook(
    localStorage,
    "removeItem",
    (k) => log("localStorage.removeItem", `Remove ${k}`),
    true,
  );
  before_hook(window.ThemeManager, "inject", function (el) {
    log(
      "ThemeManager.inject",
      `Injecting custom css variables into ${el || document.head}`,
    );
  });
  before_hook(
    window.ThemeManager,
    "init",
    () => log("ThemeManager.init", "Initializing ThemeManager"),
  );
  before_hook(
    window.ThemeManager,
    "compile",
    () => log("ThemeManager.compile", "Compiling style object into CSS"),
  );

  let renderModule_og = renderModule;
  renderModule = (...args) => {
    document.querySelector("#dbgr-path").innerText = args[0];
    return renderModule_og.apply(window, args);
  };
  const modules_num = () => {
    if (container.hidden) return;
    document.getElementById("dbgr-num-modules").innerHTML =
      document.querySelectorAll("[module]").length;
  };
  let interval = null;
  container.onclick = (e) => (
    (log("dbgr", "Hiding debugger")),
      (container.hidden = 1),
      clearInterval(interval)
  );
  window.showDbgr = () => (
    (log("dbgr", "Showing debugger")),
      (container.hidden = false),
      (interval = setInterval(modules_num, 300))
  );
  document.body.appendChild(container);
  return { renderModule };
};

const before_hook = (context, fn_name, before, proto = false) => {
  let fn_og = context[fn_name];
  let replace = proto ? context.__proto__ : context;
  replace[fn_name] = (...args) => {
    before.apply(context, args);
    return fn_og.apply(context, args);
  };
};

const after_hook = (context, fn_name, after, proto = false) => {
  let fn_og = context[fn_name];
  let replace = proto ? context.__proto__ : context;
  replace[fn_name] = (...args) => {
    const res = fn_og.apply(context, args);
    after.apply(context, [...args, res]);
    return res;
  };
};
