// TODO: functions to change theme color etc
export const ThemeManager = {
  styleEl: null,
  _style: {
    accentColor: "#ff0069",
    backgroundColor: "#121212",
    primaryTextColor: "#ffffff",
    secondaryTextColor: "#a7a7a7",
    primaryLinkColor: "#ffffff",
    secondaryLinkColor: "#5c5c5c",
    secondaryBackgroundColor: "#1e1e1e",
    successColor: "#32fc65",
    successTextColor: "#000",
    errorColor: "#fc323f",
    errorTextColor: "#fff",
    warningColor: "#fce432",
    warningTextColor: "#000000",
  },
  get style() {
    return this._style;
  },
  set style(val) {
    for (const x in val) localStorage.setItem("theme." + x, val[x]);
    return this._style = val;
  },
  serializeMaps: {
    v1: {
      accentColor: "a",
      backgroundColor: "b",
      primaryTextColor: "pt",
      secondaryTextColor: "st",
      primaryLinkColor: "pl",
      secondaryLinkColor: "sl",
      secondaryBackgroundColor: "sb",
      successColor: "sc",
      successTextColor: "st",
      errorColor: "ec",
      errorTextColor: "et",
      warningColor: "wc",
      warningTextColor: "wt",
    },
  },
  _css: null,
  get css() {
    return this._css;
  },
  set css(val) {
    this._css = val;
    this.styleEl.innerHTML = val;
    document.querySelector("meta[name='theme-color']").content =
      this.style.accentColor;
  },
  export(version) {
    let versions = {
      v1: () => {
        let serStyle = {};
        for (let key in this.serializeMaps.v1) {
          serStyle[this.serializeMaps.v1[key]] = this.style[key];
        }
        return btoa(
          JSON.stringify({ v: 1, ...serStyle }).replaceAll(/[{}"]/g, ""),
        ).replaceAll(/=/g, "");
      },
    };
    return versions[version]();
  },
  import(code) {
    let versions = {
      v1: (code) => {
        code = `{ "${atob(code).replaceAll(/([:,])/g, '"$1"')}" }`;
        const serStyle = JSON.parse(code);
        let deserStyle = {};
        for (let key in this.serializeMaps.v1) {
          deserStyle[key] = serStyle[this.serializeMaps.v1[key]];
        }
        this.style = deserStyle;
        this.compile();
        return this.style;
      },
    };
    return versions.v1(code);
  },
  init(el = null) {
    for (let prop in this.style) {
      this[prop] = `var(${this.cssifyName(prop)})`;
      this.style[prop] = localStorage.getItem("theme." + prop) ||
        this.style[prop];
    }
    this.inject(el);
    return this.compile();
  },
  cssifyName(name) {
    return "--" + name.replaceAll(/[A-Z]/g, (m) => "-" + m.toLowerCase());
  },
  compile() {
    let css = "";
    for (let prop in this.style) {
      const val = this.style[prop];
      prop = this.cssifyName(prop); 
      css += `${prop}: ${val};\n`;
    }
    this.css = `:root { ${css} }`;
    document.dispatchEvent(new Event("themeReload"));
    return css;
  },
  set(prop, val) {
    if (!this.style.hasOwnProperty(prop)) return false;
    this.style[prop] = val;
    localStorage.setItem("theme." + prop, val);
    return this.compile();
  },
  inject(el = null) {
    if (!el) el = document.head;
    this.styleEl = document.createElement("style");
    el.appendChild(this.styleEl);
  },
};
