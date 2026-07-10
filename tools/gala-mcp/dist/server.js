import Qn from "ajv";
import Kn from "ajv-formats";
import $t from "node:process";
import { z as a } from "zod";
import { spawn as Yn } from "node:child_process";
import { join as Ge } from "node:path";
import { fileURLToPath as Xn } from "node:url";
function l(e, t, n) {
  function o(c, p) {
    var d;
    Object.defineProperty(c, "_zod", {
      value: c._zod ?? {},
      enumerable: !1
    }), (d = c._zod).traits ?? (d.traits = /* @__PURE__ */ new Set()), c._zod.traits.add(e), t(c, p);
    for (const f in i.prototype)
      f in c || Object.defineProperty(c, f, { value: i.prototype[f].bind(c) });
    c._zod.constr = i, c._zod.def = p;
  }
  const r = n?.Parent ?? Object;
  class s extends r {
  }
  Object.defineProperty(s, "name", { value: e });
  function i(c) {
    var p;
    const d = n?.Parent ? new s() : this;
    o(d, c), (p = d._zod).deferred ?? (p.deferred = []);
    for (const f of d._zod.deferred)
      f();
    return d;
  }
  return Object.defineProperty(i, "init", { value: o }), Object.defineProperty(i, Symbol.hasInstance, {
    value: (c) => n?.Parent && c instanceof n.Parent ? !0 : c?._zod?.traits?.has(e)
  }), Object.defineProperty(i, "name", { value: e }), i;
}
class de extends Error {
  constructor() {
    super("Encountered Promise during synchronous parse. Use .parseAsync() instead.");
  }
}
const Bt = {};
function W(e) {
  return Bt;
}
function eo(e) {
  const t = Object.values(e).filter((o) => typeof o == "number");
  return Object.entries(e).filter(([o, r]) => t.indexOf(+o) === -1).map(([o, r]) => r);
}
function to(e, t) {
  return typeof t == "bigint" ? t.toString() : t;
}
function We(e) {
  return {
    get value() {
      {
        const t = e();
        return Object.defineProperty(this, "value", { value: t }), t;
      }
    }
  };
}
function Qe(e) {
  return e == null;
}
function Ke(e) {
  const t = e.startsWith("^") ? 1 : 0, n = e.endsWith("$") ? e.length - 1 : e.length;
  return e.slice(t, n);
}
function no(e, t) {
  const n = (e.toString().split(".")[1] || "").length, o = (t.toString().split(".")[1] || "").length, r = n > o ? n : o, s = Number.parseInt(e.toFixed(r).replace(".", "")), i = Number.parseInt(t.toFixed(r).replace(".", ""));
  return s % i / 10 ** r;
}
function z(e, t, n) {
  Object.defineProperty(e, t, {
    get() {
      {
        const o = n();
        return e[t] = o, o;
      }
    },
    set(o) {
      Object.defineProperty(e, t, {
        value: o
        // configurable: true,
      });
    },
    configurable: !0
  });
}
function Ne(e, t, n) {
  Object.defineProperty(e, t, {
    value: n,
    writable: !0,
    enumerable: !0,
    configurable: !0
  });
}
function ue(e) {
  return JSON.stringify(e);
}
const Wt = Error.captureStackTrace ? Error.captureStackTrace : (...e) => {
};
function Ee(e) {
  return typeof e == "object" && e !== null && !Array.isArray(e);
}
const oo = We(() => {
  if (typeof navigator < "u" && navigator?.userAgent?.includes("Cloudflare"))
    return !1;
  try {
    const e = Function;
    return new e(""), !0;
  } catch {
    return !1;
  }
});
function xe(e) {
  if (Ee(e) === !1)
    return !1;
  const t = e.constructor;
  if (t === void 0)
    return !0;
  const n = t.prototype;
  return !(Ee(n) === !1 || Object.prototype.hasOwnProperty.call(n, "isPrototypeOf") === !1);
}
const ro = /* @__PURE__ */ new Set(["string", "number", "symbol"]);
function he(e) {
  return e.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
function te(e, t, n) {
  const o = new e._zod.constr(t ?? e._zod.def);
  return (!t || n?.parent) && (o._zod.parent = e), o;
}
function _(e) {
  const t = e;
  if (!t)
    return {};
  if (typeof t == "string")
    return { error: () => t };
  if (t?.message !== void 0) {
    if (t?.error !== void 0)
      throw new Error("Cannot specify both `message` and `error` params");
    t.error = t.message;
  }
  return delete t.message, typeof t.error == "string" ? { ...t, error: () => t.error } : t;
}
function so(e) {
  return Object.keys(e).filter((t) => e[t]._zod.optin === "optional" && e[t]._zod.optout === "optional");
}
const io = {
  safeint: [Number.MIN_SAFE_INTEGER, Number.MAX_SAFE_INTEGER],
  int32: [-2147483648, 2147483647],
  uint32: [0, 4294967295],
  float32: [-34028234663852886e22, 34028234663852886e22],
  float64: [-Number.MAX_VALUE, Number.MAX_VALUE]
};
function ao(e, t) {
  const n = {}, o = e._zod.def;
  for (const r in t) {
    if (!(r in o.shape))
      throw new Error(`Unrecognized key: "${r}"`);
    t[r] && (n[r] = o.shape[r]);
  }
  return te(e, {
    ...e._zod.def,
    shape: n,
    checks: []
  });
}
function co(e, t) {
  const n = { ...e._zod.def.shape }, o = e._zod.def;
  for (const r in t) {
    if (!(r in o.shape))
      throw new Error(`Unrecognized key: "${r}"`);
    t[r] && delete n[r];
  }
  return te(e, {
    ...e._zod.def,
    shape: n,
    checks: []
  });
}
function uo(e, t) {
  if (!xe(t))
    throw new Error("Invalid input to extend: expected a plain object");
  const n = {
    ...e._zod.def,
    get shape() {
      const o = { ...e._zod.def.shape, ...t };
      return Ne(this, "shape", o), o;
    },
    checks: []
    // delete existing checks
  };
  return te(e, n);
}
function lo(e, t) {
  return te(e, {
    ...e._zod.def,
    get shape() {
      const n = { ...e._zod.def.shape, ...t._zod.def.shape };
      return Ne(this, "shape", n), n;
    },
    catchall: t._zod.def.catchall,
    checks: []
    // delete existing checks
  });
}
function po(e, t, n) {
  const o = t._zod.def.shape, r = { ...o };
  if (n)
    for (const s in n) {
      if (!(s in o))
        throw new Error(`Unrecognized key: "${s}"`);
      n[s] && (r[s] = e ? new e({
        type: "optional",
        innerType: o[s]
      }) : o[s]);
    }
  else
    for (const s in o)
      r[s] = e ? new e({
        type: "optional",
        innerType: o[s]
      }) : o[s];
  return te(t, {
    ...t._zod.def,
    shape: r,
    checks: []
  });
}
function mo(e, t, n) {
  const o = t._zod.def.shape, r = { ...o };
  if (n)
    for (const s in n) {
      if (!(s in r))
        throw new Error(`Unrecognized key: "${s}"`);
      n[s] && (r[s] = new e({
        type: "nonoptional",
        innerType: o[s]
      }));
    }
  else
    for (const s in o)
      r[s] = new e({
        type: "nonoptional",
        innerType: o[s]
      });
  return te(t, {
    ...t._zod.def,
    shape: r,
    // optional: [],
    checks: []
  });
}
function le(e, t = 0) {
  for (let n = t; n < e.issues.length; n++)
    if (e.issues[n]?.continue !== !0)
      return !0;
  return !1;
}
function ee(e, t) {
  return t.map((n) => {
    var o;
    return (o = n).path ?? (o.path = []), n.path.unshift(e), n;
  });
}
function Se(e) {
  return typeof e == "string" ? e : e?.message;
}
function Q(e, t, n) {
  const o = { ...e, path: e.path ?? [] };
  if (!e.message) {
    const r = Se(e.inst?._zod.def?.error?.(e)) ?? Se(t?.error?.(e)) ?? Se(n.customError?.(e)) ?? Se(n.localeError?.(e)) ?? "Invalid input";
    o.message = r;
  }
  return delete o.inst, delete o.continue, t?.reportInput || delete o.input, o;
}
function Ye(e) {
  return Array.isArray(e) ? "array" : typeof e == "string" ? "string" : "unknown";
}
function me(...e) {
  const [t, n, o] = e;
  return typeof t == "string" ? {
    message: t,
    code: "custom",
    input: n,
    inst: o
  } : { ...t };
}
const Qt = (e, t) => {
  e.name = "$ZodError", Object.defineProperty(e, "_zod", {
    value: e._zod,
    enumerable: !1
  }), Object.defineProperty(e, "issues", {
    value: t,
    enumerable: !1
  }), Object.defineProperty(e, "message", {
    get() {
      return JSON.stringify(t, to, 2);
    },
    enumerable: !0
    // configurable: false,
  }), Object.defineProperty(e, "toString", {
    value: () => e.message,
    enumerable: !1
  });
}, Kt = l("$ZodError", Qt), Yt = l("$ZodError", Qt, { Parent: Error });
function ho(e, t = (n) => n.message) {
  const n = {}, o = [];
  for (const r of e.issues)
    r.path.length > 0 ? (n[r.path[0]] = n[r.path[0]] || [], n[r.path[0]].push(t(r))) : o.push(t(r));
  return { formErrors: o, fieldErrors: n };
}
function fo(e, t) {
  const n = t || function(s) {
    return s.message;
  }, o = { _errors: [] }, r = (s) => {
    for (const i of s.issues)
      if (i.code === "invalid_union" && i.errors.length)
        i.errors.map((c) => r({ issues: c }));
      else if (i.code === "invalid_key")
        r({ issues: i.issues });
      else if (i.code === "invalid_element")
        r({ issues: i.issues });
      else if (i.path.length === 0)
        o._errors.push(n(i));
      else {
        let c = o, p = 0;
        for (; p < i.path.length; ) {
          const d = i.path[p];
          p === i.path.length - 1 ? (c[d] = c[d] || { _errors: [] }, c[d]._errors.push(n(i))) : c[d] = c[d] || { _errors: [] }, c = c[d], p++;
        }
      }
  };
  return r(e), o;
}
const go = (e) => (t, n, o, r) => {
  const s = o ? Object.assign(o, { async: !1 }) : { async: !1 }, i = t._zod.run({ value: n, issues: [] }, s);
  if (i instanceof Promise)
    throw new de();
  if (i.issues.length) {
    const c = new (r?.Err ?? e)(i.issues.map((p) => Q(p, s, W())));
    throw Wt(c, r?.callee), c;
  }
  return i.value;
}, _o = (e) => async (t, n, o, r) => {
  const s = o ? Object.assign(o, { async: !0 }) : { async: !0 };
  let i = t._zod.run({ value: n, issues: [] }, s);
  if (i instanceof Promise && (i = await i), i.issues.length) {
    const c = new (r?.Err ?? e)(i.issues.map((p) => Q(p, s, W())));
    throw Wt(c, r?.callee), c;
  }
  return i.value;
}, Xt = (e) => (t, n, o) => {
  const r = o ? { ...o, async: !1 } : { async: !1 }, s = t._zod.run({ value: n, issues: [] }, r);
  if (s instanceof Promise)
    throw new de();
  return s.issues.length ? {
    success: !1,
    error: new (e ?? Kt)(s.issues.map((i) => Q(i, r, W())))
  } : { success: !0, data: s.value };
}, en = /* @__PURE__ */ Xt(Yt), tn = (e) => async (t, n, o) => {
  const r = o ? Object.assign(o, { async: !0 }) : { async: !0 };
  let s = t._zod.run({ value: n, issues: [] }, r);
  return s instanceof Promise && (s = await s), s.issues.length ? {
    success: !1,
    error: new e(s.issues.map((i) => Q(i, r, W())))
  } : { success: !0, data: s.value };
}, vo = /* @__PURE__ */ tn(Yt), bo = /^[cC][^\s-]{8,}$/, wo = /^[0-9a-z]+$/, ko = /^[0-9A-HJKMNP-TV-Za-hjkmnp-tv-z]{26}$/, yo = /^[0-9a-vA-V]{20}$/, So = /^[A-Za-z0-9]{27}$/, To = /^[a-zA-Z0-9_-]{21}$/, $o = /^P(?:(\d+W)|(?!.*W)(?=\d|T\d)(\d+Y)?(\d+M)?(\d+D)?(T(?=\d)(\d+H)?(\d+M)?(\d+([.,]\d+)?S)?)?)$/, Ro = /^([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})$/, Rt = (e) => e ? new RegExp(`^([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-${e}[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12})$`) : /^([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-8][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}|00000000-0000-0000-0000-000000000000)$/, zo = /^(?!\.)(?!.*\.\.)([A-Za-z0-9_'+\-\.]*)[A-Za-z0-9_+-]@([A-Za-z0-9][A-Za-z0-9\-]*\.)+[A-Za-z]{2,}$/, Io = "^(\\p{Extended_Pictographic}|\\p{Emoji_Component})+$";
function Eo() {
  return new RegExp(Io, "u");
}
const xo = /^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$/, Zo = /^(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|::|([0-9a-fA-F]{1,4})?::([0-9a-fA-F]{1,4}:?){0,6})$/, Po = /^((25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\/([0-9]|[1-2][0-9]|3[0-2])$/, qo = /^(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|::|([0-9a-fA-F]{1,4})?::([0-9a-fA-F]{1,4}:?){0,6})\/(12[0-8]|1[01][0-9]|[1-9]?[0-9])$/, Co = /^$|^(?:[0-9a-zA-Z+/]{4})*(?:(?:[0-9a-zA-Z+/]{2}==)|(?:[0-9a-zA-Z+/]{3}=))?$/, nn = /^[A-Za-z0-9_-]*$/, jo = /^([a-zA-Z0-9-]+\.)*[a-zA-Z0-9-]+$/, No = /^\+(?:[0-9]){6,14}[0-9]$/, on = "(?:(?:\\d\\d[2468][048]|\\d\\d[13579][26]|\\d\\d0[48]|[02468][048]00|[13579][26]00)-02-29|\\d{4}-(?:(?:0[13578]|1[02])-(?:0[1-9]|[12]\\d|3[01])|(?:0[469]|11)-(?:0[1-9]|[12]\\d|30)|(?:02)-(?:0[1-9]|1\\d|2[0-8])))", Oo = /* @__PURE__ */ new RegExp(`^${on}$`);
function rn(e) {
  const t = "(?:[01]\\d|2[0-3]):[0-5]\\d";
  return typeof e.precision == "number" ? e.precision === -1 ? `${t}` : e.precision === 0 ? `${t}:[0-5]\\d` : `${t}:[0-5]\\d\\.\\d{${e.precision}}` : `${t}(?::[0-5]\\d(?:\\.\\d+)?)?`;
}
function Ao(e) {
  return new RegExp(`^${rn(e)}$`);
}
function Mo(e) {
  const t = rn({ precision: e.precision }), n = ["Z"];
  e.local && n.push(""), e.offset && n.push("([+-]\\d{2}:\\d{2})");
  const o = `${t}(?:${n.join("|")})`;
  return new RegExp(`^${on}T(?:${o})$`);
}
const Lo = (e) => {
  const t = e ? `[\\s\\S]{${e?.minimum ?? 0},${e?.maximum ?? ""}}` : "[\\s\\S]*";
  return new RegExp(`^${t}$`);
}, Uo = /^\d+$/, Do = /^-?\d+(?:\.\d+)?/i, Fo = /true|false/i, Ho = /null/i, Go = /^[^A-Z]*$/, Vo = /^[^a-z]*$/, F = /* @__PURE__ */ l("$ZodCheck", (e, t) => {
  var n;
  e._zod ?? (e._zod = {}), e._zod.def = t, (n = e._zod).onattach ?? (n.onattach = []);
}), sn = {
  number: "number",
  bigint: "bigint",
  object: "date"
}, an = /* @__PURE__ */ l("$ZodCheckLessThan", (e, t) => {
  F.init(e, t);
  const n = sn[typeof t.value];
  e._zod.onattach.push((o) => {
    const r = o._zod.bag, s = (t.inclusive ? r.maximum : r.exclusiveMaximum) ?? Number.POSITIVE_INFINITY;
    t.value < s && (t.inclusive ? r.maximum = t.value : r.exclusiveMaximum = t.value);
  }), e._zod.check = (o) => {
    (t.inclusive ? o.value <= t.value : o.value < t.value) || o.issues.push({
      origin: n,
      code: "too_big",
      maximum: t.value,
      input: o.value,
      inclusive: t.inclusive,
      inst: e,
      continue: !t.abort
    });
  };
}), cn = /* @__PURE__ */ l("$ZodCheckGreaterThan", (e, t) => {
  F.init(e, t);
  const n = sn[typeof t.value];
  e._zod.onattach.push((o) => {
    const r = o._zod.bag, s = (t.inclusive ? r.minimum : r.exclusiveMinimum) ?? Number.NEGATIVE_INFINITY;
    t.value > s && (t.inclusive ? r.minimum = t.value : r.exclusiveMinimum = t.value);
  }), e._zod.check = (o) => {
    (t.inclusive ? o.value >= t.value : o.value > t.value) || o.issues.push({
      origin: n,
      code: "too_small",
      minimum: t.value,
      input: o.value,
      inclusive: t.inclusive,
      inst: e,
      continue: !t.abort
    });
  };
}), Jo = /* @__PURE__ */ l("$ZodCheckMultipleOf", (e, t) => {
  F.init(e, t), e._zod.onattach.push((n) => {
    var o;
    (o = n._zod.bag).multipleOf ?? (o.multipleOf = t.value);
  }), e._zod.check = (n) => {
    if (typeof n.value != typeof t.value)
      throw new Error("Cannot mix number and bigint in multiple_of check.");
    (typeof n.value == "bigint" ? n.value % t.value === BigInt(0) : no(n.value, t.value) === 0) || n.issues.push({
      origin: typeof n.value,
      code: "not_multiple_of",
      divisor: t.value,
      input: n.value,
      inst: e,
      continue: !t.abort
    });
  };
}), Bo = /* @__PURE__ */ l("$ZodCheckNumberFormat", (e, t) => {
  F.init(e, t), t.format = t.format || "float64";
  const n = t.format?.includes("int"), o = n ? "int" : "number", [r, s] = io[t.format];
  e._zod.onattach.push((i) => {
    const c = i._zod.bag;
    c.format = t.format, c.minimum = r, c.maximum = s, n && (c.pattern = Uo);
  }), e._zod.check = (i) => {
    const c = i.value;
    if (n) {
      if (!Number.isInteger(c)) {
        i.issues.push({
          expected: o,
          format: t.format,
          code: "invalid_type",
          input: c,
          inst: e
        });
        return;
      }
      if (!Number.isSafeInteger(c)) {
        c > 0 ? i.issues.push({
          input: c,
          code: "too_big",
          maximum: Number.MAX_SAFE_INTEGER,
          note: "Integers must be within the safe integer range.",
          inst: e,
          origin: o,
          continue: !t.abort
        }) : i.issues.push({
          input: c,
          code: "too_small",
          minimum: Number.MIN_SAFE_INTEGER,
          note: "Integers must be within the safe integer range.",
          inst: e,
          origin: o,
          continue: !t.abort
        });
        return;
      }
    }
    c < r && i.issues.push({
      origin: "number",
      input: c,
      code: "too_small",
      minimum: r,
      inclusive: !0,
      inst: e,
      continue: !t.abort
    }), c > s && i.issues.push({
      origin: "number",
      input: c,
      code: "too_big",
      maximum: s,
      inst: e
    });
  };
}), Wo = /* @__PURE__ */ l("$ZodCheckMaxLength", (e, t) => {
  var n;
  F.init(e, t), (n = e._zod.def).when ?? (n.when = (o) => {
    const r = o.value;
    return !Qe(r) && r.length !== void 0;
  }), e._zod.onattach.push((o) => {
    const r = o._zod.bag.maximum ?? Number.POSITIVE_INFINITY;
    t.maximum < r && (o._zod.bag.maximum = t.maximum);
  }), e._zod.check = (o) => {
    const r = o.value;
    if (r.length <= t.maximum)
      return;
    const i = Ye(r);
    o.issues.push({
      origin: i,
      code: "too_big",
      maximum: t.maximum,
      inclusive: !0,
      input: r,
      inst: e,
      continue: !t.abort
    });
  };
}), Qo = /* @__PURE__ */ l("$ZodCheckMinLength", (e, t) => {
  var n;
  F.init(e, t), (n = e._zod.def).when ?? (n.when = (o) => {
    const r = o.value;
    return !Qe(r) && r.length !== void 0;
  }), e._zod.onattach.push((o) => {
    const r = o._zod.bag.minimum ?? Number.NEGATIVE_INFINITY;
    t.minimum > r && (o._zod.bag.minimum = t.minimum);
  }), e._zod.check = (o) => {
    const r = o.value;
    if (r.length >= t.minimum)
      return;
    const i = Ye(r);
    o.issues.push({
      origin: i,
      code: "too_small",
      minimum: t.minimum,
      inclusive: !0,
      input: r,
      inst: e,
      continue: !t.abort
    });
  };
}), Ko = /* @__PURE__ */ l("$ZodCheckLengthEquals", (e, t) => {
  var n;
  F.init(e, t), (n = e._zod.def).when ?? (n.when = (o) => {
    const r = o.value;
    return !Qe(r) && r.length !== void 0;
  }), e._zod.onattach.push((o) => {
    const r = o._zod.bag;
    r.minimum = t.length, r.maximum = t.length, r.length = t.length;
  }), e._zod.check = (o) => {
    const r = o.value, s = r.length;
    if (s === t.length)
      return;
    const i = Ye(r), c = s > t.length;
    o.issues.push({
      origin: i,
      ...c ? { code: "too_big", maximum: t.length } : { code: "too_small", minimum: t.length },
      inclusive: !0,
      exact: !0,
      input: o.value,
      inst: e,
      continue: !t.abort
    });
  };
}), Oe = /* @__PURE__ */ l("$ZodCheckStringFormat", (e, t) => {
  var n, o;
  F.init(e, t), e._zod.onattach.push((r) => {
    const s = r._zod.bag;
    s.format = t.format, t.pattern && (s.patterns ?? (s.patterns = /* @__PURE__ */ new Set()), s.patterns.add(t.pattern));
  }), t.pattern ? (n = e._zod).check ?? (n.check = (r) => {
    t.pattern.lastIndex = 0, !t.pattern.test(r.value) && r.issues.push({
      origin: "string",
      code: "invalid_format",
      format: t.format,
      input: r.value,
      ...t.pattern ? { pattern: t.pattern.toString() } : {},
      inst: e,
      continue: !t.abort
    });
  }) : (o = e._zod).check ?? (o.check = () => {
  });
}), Yo = /* @__PURE__ */ l("$ZodCheckRegex", (e, t) => {
  Oe.init(e, t), e._zod.check = (n) => {
    t.pattern.lastIndex = 0, !t.pattern.test(n.value) && n.issues.push({
      origin: "string",
      code: "invalid_format",
      format: "regex",
      input: n.value,
      pattern: t.pattern.toString(),
      inst: e,
      continue: !t.abort
    });
  };
}), Xo = /* @__PURE__ */ l("$ZodCheckLowerCase", (e, t) => {
  t.pattern ?? (t.pattern = Go), Oe.init(e, t);
}), er = /* @__PURE__ */ l("$ZodCheckUpperCase", (e, t) => {
  t.pattern ?? (t.pattern = Vo), Oe.init(e, t);
}), tr = /* @__PURE__ */ l("$ZodCheckIncludes", (e, t) => {
  F.init(e, t);
  const n = he(t.includes), o = new RegExp(typeof t.position == "number" ? `^.{${t.position}}${n}` : n);
  t.pattern = o, e._zod.onattach.push((r) => {
    const s = r._zod.bag;
    s.patterns ?? (s.patterns = /* @__PURE__ */ new Set()), s.patterns.add(o);
  }), e._zod.check = (r) => {
    r.value.includes(t.includes, t.position) || r.issues.push({
      origin: "string",
      code: "invalid_format",
      format: "includes",
      includes: t.includes,
      input: r.value,
      inst: e,
      continue: !t.abort
    });
  };
}), nr = /* @__PURE__ */ l("$ZodCheckStartsWith", (e, t) => {
  F.init(e, t);
  const n = new RegExp(`^${he(t.prefix)}.*`);
  t.pattern ?? (t.pattern = n), e._zod.onattach.push((o) => {
    const r = o._zod.bag;
    r.patterns ?? (r.patterns = /* @__PURE__ */ new Set()), r.patterns.add(n);
  }), e._zod.check = (o) => {
    o.value.startsWith(t.prefix) || o.issues.push({
      origin: "string",
      code: "invalid_format",
      format: "starts_with",
      prefix: t.prefix,
      input: o.value,
      inst: e,
      continue: !t.abort
    });
  };
}), or = /* @__PURE__ */ l("$ZodCheckEndsWith", (e, t) => {
  F.init(e, t);
  const n = new RegExp(`.*${he(t.suffix)}$`);
  t.pattern ?? (t.pattern = n), e._zod.onattach.push((o) => {
    const r = o._zod.bag;
    r.patterns ?? (r.patterns = /* @__PURE__ */ new Set()), r.patterns.add(n);
  }), e._zod.check = (o) => {
    o.value.endsWith(t.suffix) || o.issues.push({
      origin: "string",
      code: "invalid_format",
      format: "ends_with",
      suffix: t.suffix,
      input: o.value,
      inst: e,
      continue: !t.abort
    });
  };
}), rr = /* @__PURE__ */ l("$ZodCheckOverwrite", (e, t) => {
  F.init(e, t), e._zod.check = (n) => {
    n.value = t.tx(n.value);
  };
});
class sr {
  constructor(t = []) {
    this.content = [], this.indent = 0, this && (this.args = t);
  }
  indented(t) {
    this.indent += 1, t(this), this.indent -= 1;
  }
  write(t) {
    if (typeof t == "function") {
      t(this, { execution: "sync" }), t(this, { execution: "async" });
      return;
    }
    const o = t.split(`
`).filter((i) => i), r = Math.min(...o.map((i) => i.length - i.trimStart().length)), s = o.map((i) => i.slice(r)).map((i) => " ".repeat(this.indent * 2) + i);
    for (const i of s)
      this.content.push(i);
  }
  compile() {
    const t = Function, n = this?.args, r = [...(this?.content ?? [""]).map((s) => `  ${s}`)];
    return new t(...n, r.join(`
`));
  }
}
const ir = {
  major: 4,
  minor: 0,
  patch: 0
}, E = /* @__PURE__ */ l("$ZodType", (e, t) => {
  var n;
  e ?? (e = {}), e._zod.def = t, e._zod.bag = e._zod.bag || {}, e._zod.version = ir;
  const o = [...e._zod.def.checks ?? []];
  e._zod.traits.has("$ZodCheck") && o.unshift(e);
  for (const r of o)
    for (const s of r._zod.onattach)
      s(e);
  if (o.length === 0)
    (n = e._zod).deferred ?? (n.deferred = []), e._zod.deferred?.push(() => {
      e._zod.run = e._zod.parse;
    });
  else {
    const r = (s, i, c) => {
      let p = le(s), d;
      for (const f of i) {
        if (f._zod.def.when) {
          if (!f._zod.def.when(s))
            continue;
        } else if (p)
          continue;
        const m = s.issues.length, g = f._zod.check(s);
        if (g instanceof Promise && c?.async === !1)
          throw new de();
        if (d || g instanceof Promise)
          d = (d ?? Promise.resolve()).then(async () => {
            await g, s.issues.length !== m && (p || (p = le(s, m)));
          });
        else {
          if (s.issues.length === m)
            continue;
          p || (p = le(s, m));
        }
      }
      return d ? d.then(() => s) : s;
    };
    e._zod.run = (s, i) => {
      const c = e._zod.parse(s, i);
      if (c instanceof Promise) {
        if (i.async === !1)
          throw new de();
        return c.then((p) => r(p, o, i));
      }
      return r(c, o, i);
    };
  }
  e["~standard"] = {
    validate: (r) => {
      try {
        const s = en(e, r);
        return s.success ? { value: s.data } : { issues: s.error?.issues };
      } catch {
        return vo(e, r).then((i) => i.success ? { value: i.data } : { issues: i.error?.issues });
      }
    },
    vendor: "zod",
    version: 1
  };
}), Xe = /* @__PURE__ */ l("$ZodString", (e, t) => {
  E.init(e, t), e._zod.pattern = [...e?._zod.bag?.patterns ?? []].pop() ?? Lo(e._zod.bag), e._zod.parse = (n, o) => {
    if (t.coerce)
      try {
        n.value = String(n.value);
      } catch {
      }
    return typeof n.value == "string" || n.issues.push({
      expected: "string",
      code: "invalid_type",
      input: n.value,
      inst: e
    }), n;
  };
}), Z = /* @__PURE__ */ l("$ZodStringFormat", (e, t) => {
  Oe.init(e, t), Xe.init(e, t);
}), ar = /* @__PURE__ */ l("$ZodGUID", (e, t) => {
  t.pattern ?? (t.pattern = Ro), Z.init(e, t);
}), cr = /* @__PURE__ */ l("$ZodUUID", (e, t) => {
  if (t.version) {
    const o = {
      v1: 1,
      v2: 2,
      v3: 3,
      v4: 4,
      v5: 5,
      v6: 6,
      v7: 7,
      v8: 8
    }[t.version];
    if (o === void 0)
      throw new Error(`Invalid UUID version: "${t.version}"`);
    t.pattern ?? (t.pattern = Rt(o));
  } else
    t.pattern ?? (t.pattern = Rt());
  Z.init(e, t);
}), ur = /* @__PURE__ */ l("$ZodEmail", (e, t) => {
  t.pattern ?? (t.pattern = zo), Z.init(e, t);
}), lr = /* @__PURE__ */ l("$ZodURL", (e, t) => {
  Z.init(e, t), e._zod.check = (n) => {
    try {
      const o = n.value, r = new URL(o), s = r.href;
      t.hostname && (t.hostname.lastIndex = 0, t.hostname.test(r.hostname) || n.issues.push({
        code: "invalid_format",
        format: "url",
        note: "Invalid hostname",
        pattern: jo.source,
        input: n.value,
        inst: e,
        continue: !t.abort
      })), t.protocol && (t.protocol.lastIndex = 0, t.protocol.test(r.protocol.endsWith(":") ? r.protocol.slice(0, -1) : r.protocol) || n.issues.push({
        code: "invalid_format",
        format: "url",
        note: "Invalid protocol",
        pattern: t.protocol.source,
        input: n.value,
        inst: e,
        continue: !t.abort
      })), !o.endsWith("/") && s.endsWith("/") ? n.value = s.slice(0, -1) : n.value = s;
      return;
    } catch {
      n.issues.push({
        code: "invalid_format",
        format: "url",
        input: n.value,
        inst: e,
        continue: !t.abort
      });
    }
  };
}), pr = /* @__PURE__ */ l("$ZodEmoji", (e, t) => {
  t.pattern ?? (t.pattern = Eo()), Z.init(e, t);
}), dr = /* @__PURE__ */ l("$ZodNanoID", (e, t) => {
  t.pattern ?? (t.pattern = To), Z.init(e, t);
}), mr = /* @__PURE__ */ l("$ZodCUID", (e, t) => {
  t.pattern ?? (t.pattern = bo), Z.init(e, t);
}), hr = /* @__PURE__ */ l("$ZodCUID2", (e, t) => {
  t.pattern ?? (t.pattern = wo), Z.init(e, t);
}), fr = /* @__PURE__ */ l("$ZodULID", (e, t) => {
  t.pattern ?? (t.pattern = ko), Z.init(e, t);
}), gr = /* @__PURE__ */ l("$ZodXID", (e, t) => {
  t.pattern ?? (t.pattern = yo), Z.init(e, t);
}), _r = /* @__PURE__ */ l("$ZodKSUID", (e, t) => {
  t.pattern ?? (t.pattern = So), Z.init(e, t);
}), vr = /* @__PURE__ */ l("$ZodISODateTime", (e, t) => {
  t.pattern ?? (t.pattern = Mo(t)), Z.init(e, t);
}), br = /* @__PURE__ */ l("$ZodISODate", (e, t) => {
  t.pattern ?? (t.pattern = Oo), Z.init(e, t);
}), wr = /* @__PURE__ */ l("$ZodISOTime", (e, t) => {
  t.pattern ?? (t.pattern = Ao(t)), Z.init(e, t);
}), kr = /* @__PURE__ */ l("$ZodISODuration", (e, t) => {
  t.pattern ?? (t.pattern = $o), Z.init(e, t);
}), yr = /* @__PURE__ */ l("$ZodIPv4", (e, t) => {
  t.pattern ?? (t.pattern = xo), Z.init(e, t), e._zod.onattach.push((n) => {
    const o = n._zod.bag;
    o.format = "ipv4";
  });
}), Sr = /* @__PURE__ */ l("$ZodIPv6", (e, t) => {
  t.pattern ?? (t.pattern = Zo), Z.init(e, t), e._zod.onattach.push((n) => {
    const o = n._zod.bag;
    o.format = "ipv6";
  }), e._zod.check = (n) => {
    try {
      new URL(`http://[${n.value}]`);
    } catch {
      n.issues.push({
        code: "invalid_format",
        format: "ipv6",
        input: n.value,
        inst: e,
        continue: !t.abort
      });
    }
  };
}), Tr = /* @__PURE__ */ l("$ZodCIDRv4", (e, t) => {
  t.pattern ?? (t.pattern = Po), Z.init(e, t);
}), $r = /* @__PURE__ */ l("$ZodCIDRv6", (e, t) => {
  t.pattern ?? (t.pattern = qo), Z.init(e, t), e._zod.check = (n) => {
    const [o, r] = n.value.split("/");
    try {
      if (!r)
        throw new Error();
      const s = Number(r);
      if (`${s}` !== r)
        throw new Error();
      if (s < 0 || s > 128)
        throw new Error();
      new URL(`http://[${o}]`);
    } catch {
      n.issues.push({
        code: "invalid_format",
        format: "cidrv6",
        input: n.value,
        inst: e,
        continue: !t.abort
      });
    }
  };
});
function un(e) {
  if (e === "")
    return !0;
  if (e.length % 4 !== 0)
    return !1;
  try {
    return atob(e), !0;
  } catch {
    return !1;
  }
}
const Rr = /* @__PURE__ */ l("$ZodBase64", (e, t) => {
  t.pattern ?? (t.pattern = Co), Z.init(e, t), e._zod.onattach.push((n) => {
    n._zod.bag.contentEncoding = "base64";
  }), e._zod.check = (n) => {
    un(n.value) || n.issues.push({
      code: "invalid_format",
      format: "base64",
      input: n.value,
      inst: e,
      continue: !t.abort
    });
  };
});
function zr(e) {
  if (!nn.test(e))
    return !1;
  const t = e.replace(/[-_]/g, (o) => o === "-" ? "+" : "/"), n = t.padEnd(Math.ceil(t.length / 4) * 4, "=");
  return un(n);
}
const Ir = /* @__PURE__ */ l("$ZodBase64URL", (e, t) => {
  t.pattern ?? (t.pattern = nn), Z.init(e, t), e._zod.onattach.push((n) => {
    n._zod.bag.contentEncoding = "base64url";
  }), e._zod.check = (n) => {
    zr(n.value) || n.issues.push({
      code: "invalid_format",
      format: "base64url",
      input: n.value,
      inst: e,
      continue: !t.abort
    });
  };
}), Er = /* @__PURE__ */ l("$ZodE164", (e, t) => {
  t.pattern ?? (t.pattern = No), Z.init(e, t);
});
function xr(e, t = null) {
  try {
    const n = e.split(".");
    if (n.length !== 3)
      return !1;
    const [o] = n;
    if (!o)
      return !1;
    const r = JSON.parse(atob(o));
    return !("typ" in r && r?.typ !== "JWT" || !r.alg || t && (!("alg" in r) || r.alg !== t));
  } catch {
    return !1;
  }
}
const Zr = /* @__PURE__ */ l("$ZodJWT", (e, t) => {
  Z.init(e, t), e._zod.check = (n) => {
    xr(n.value, t.alg) || n.issues.push({
      code: "invalid_format",
      format: "jwt",
      input: n.value,
      inst: e,
      continue: !t.abort
    });
  };
}), ln = /* @__PURE__ */ l("$ZodNumber", (e, t) => {
  E.init(e, t), e._zod.pattern = e._zod.bag.pattern ?? Do, e._zod.parse = (n, o) => {
    if (t.coerce)
      try {
        n.value = Number(n.value);
      } catch {
      }
    const r = n.value;
    if (typeof r == "number" && !Number.isNaN(r) && Number.isFinite(r))
      return n;
    const s = typeof r == "number" ? Number.isNaN(r) ? "NaN" : Number.isFinite(r) ? void 0 : "Infinity" : void 0;
    return n.issues.push({
      expected: "number",
      code: "invalid_type",
      input: r,
      inst: e,
      ...s ? { received: s } : {}
    }), n;
  };
}), Pr = /* @__PURE__ */ l("$ZodNumber", (e, t) => {
  Bo.init(e, t), ln.init(e, t);
}), qr = /* @__PURE__ */ l("$ZodBoolean", (e, t) => {
  E.init(e, t), e._zod.pattern = Fo, e._zod.parse = (n, o) => {
    if (t.coerce)
      try {
        n.value = !!n.value;
      } catch {
      }
    const r = n.value;
    return typeof r == "boolean" || n.issues.push({
      expected: "boolean",
      code: "invalid_type",
      input: r,
      inst: e
    }), n;
  };
}), Cr = /* @__PURE__ */ l("$ZodNull", (e, t) => {
  E.init(e, t), e._zod.pattern = Ho, e._zod.values = /* @__PURE__ */ new Set([null]), e._zod.parse = (n, o) => {
    const r = n.value;
    return r === null || n.issues.push({
      expected: "null",
      code: "invalid_type",
      input: r,
      inst: e
    }), n;
  };
}), jr = /* @__PURE__ */ l("$ZodUnknown", (e, t) => {
  E.init(e, t), e._zod.parse = (n) => n;
}), Nr = /* @__PURE__ */ l("$ZodNever", (e, t) => {
  E.init(e, t), e._zod.parse = (n, o) => (n.issues.push({
    expected: "never",
    code: "invalid_type",
    input: n.value,
    inst: e
  }), n);
});
function zt(e, t, n) {
  e.issues.length && t.issues.push(...ee(n, e.issues)), t.value[n] = e.value;
}
const Or = /* @__PURE__ */ l("$ZodArray", (e, t) => {
  E.init(e, t), e._zod.parse = (n, o) => {
    const r = n.value;
    if (!Array.isArray(r))
      return n.issues.push({
        expected: "array",
        code: "invalid_type",
        input: r,
        inst: e
      }), n;
    n.value = Array(r.length);
    const s = [];
    for (let i = 0; i < r.length; i++) {
      const c = r[i], p = t.element._zod.run({
        value: c,
        issues: []
      }, o);
      p instanceof Promise ? s.push(p.then((d) => zt(d, n, i))) : zt(p, n, i);
    }
    return s.length ? Promise.all(s).then(() => n) : n;
  };
});
function Te(e, t, n) {
  e.issues.length && t.issues.push(...ee(n, e.issues)), t.value[n] = e.value;
}
function It(e, t, n, o) {
  e.issues.length ? o[n] === void 0 ? n in o ? t.value[n] = void 0 : t.value[n] = e.value : t.issues.push(...ee(n, e.issues)) : e.value === void 0 ? n in o && (t.value[n] = void 0) : t.value[n] = e.value;
}
const Ar = /* @__PURE__ */ l("$ZodObject", (e, t) => {
  E.init(e, t);
  const n = We(() => {
    const m = Object.keys(t.shape);
    for (const b of m)
      if (!(t.shape[b] instanceof E))
        throw new Error(`Invalid element at key "${b}": expected a Zod schema`);
    const g = so(t.shape);
    return {
      shape: t.shape,
      keys: m,
      keySet: new Set(m),
      numKeys: m.length,
      optionalKeys: new Set(g)
    };
  });
  z(e._zod, "propValues", () => {
    const m = t.shape, g = {};
    for (const b in m) {
      const R = m[b]._zod;
      if (R.values) {
        g[b] ?? (g[b] = /* @__PURE__ */ new Set());
        for (const U of R.values)
          g[b].add(U);
      }
    }
    return g;
  });
  const o = (m) => {
    const g = new sr(["shape", "payload", "ctx"]), b = n.value, R = (y) => {
      const k = ue(y);
      return `shape[${k}]._zod.run({ value: input[${k}], issues: [] }, ctx)`;
    };
    g.write("const input = payload.value;");
    const U = /* @__PURE__ */ Object.create(null);
    let ae = 0;
    for (const y of b.keys)
      U[y] = `key_${ae++}`;
    g.write("const newResult = {}");
    for (const y of b.keys)
      if (b.optionalKeys.has(y)) {
        const k = U[y];
        g.write(`const ${k} = ${R(y)};`);
        const x = ue(y);
        g.write(`
        if (${k}.issues.length) {
          if (input[${x}] === undefined) {
            if (${x} in input) {
              newResult[${x}] = undefined;
            }
          } else {
            payload.issues = payload.issues.concat(
              ${k}.issues.map((iss) => ({
                ...iss,
                path: iss.path ? [${x}, ...iss.path] : [${x}],
              }))
            );
          }
        } else if (${k}.value === undefined) {
          if (${x} in input) newResult[${x}] = undefined;
        } else {
          newResult[${x}] = ${k}.value;
        }
        `);
      } else {
        const k = U[y];
        g.write(`const ${k} = ${R(y)};`), g.write(`
          if (${k}.issues.length) payload.issues = payload.issues.concat(${k}.issues.map(iss => ({
            ...iss,
            path: iss.path ? [${ue(y)}, ...iss.path] : [${ue(y)}]
          })));`), g.write(`newResult[${ue(y)}] = ${k}.value`);
      }
    g.write("payload.value = newResult;"), g.write("return payload;");
    const K = g.compile();
    return (y, k) => K(m, y, k);
  };
  let r;
  const s = Ee, i = !Bt.jitless, p = i && oo.value, d = t.catchall;
  let f;
  e._zod.parse = (m, g) => {
    f ?? (f = n.value);
    const b = m.value;
    if (!s(b))
      return m.issues.push({
        expected: "object",
        code: "invalid_type",
        input: b,
        inst: e
      }), m;
    const R = [];
    if (i && p && g?.async === !1 && g.jitless !== !0)
      r || (r = o(t.shape)), m = r(m, g);
    else {
      m.value = {};
      const k = f.shape;
      for (const x of f.keys) {
        const ce = k[x], ye = ce._zod.run({ value: b[x], issues: [] }, g), St = ce._zod.optin === "optional" && ce._zod.optout === "optional";
        ye instanceof Promise ? R.push(ye.then((Tt) => St ? It(Tt, m, x, b) : Te(Tt, m, x))) : St ? It(ye, m, x, b) : Te(ye, m, x);
      }
    }
    if (!d)
      return R.length ? Promise.all(R).then(() => m) : m;
    const U = [], ae = f.keySet, K = d._zod, y = K.def.type;
    for (const k of Object.keys(b)) {
      if (ae.has(k))
        continue;
      if (y === "never") {
        U.push(k);
        continue;
      }
      const x = K.run({ value: b[k], issues: [] }, g);
      x instanceof Promise ? R.push(x.then((ce) => Te(ce, m, k))) : Te(x, m, k);
    }
    return U.length && m.issues.push({
      code: "unrecognized_keys",
      keys: U,
      input: b,
      inst: e
    }), R.length ? Promise.all(R).then(() => m) : m;
  };
});
function Et(e, t, n, o) {
  for (const r of e)
    if (r.issues.length === 0)
      return t.value = r.value, t;
  return t.issues.push({
    code: "invalid_union",
    input: t.value,
    inst: n,
    errors: e.map((r) => r.issues.map((s) => Q(s, o, W())))
  }), t;
}
const pn = /* @__PURE__ */ l("$ZodUnion", (e, t) => {
  E.init(e, t), z(e._zod, "optin", () => t.options.some((n) => n._zod.optin === "optional") ? "optional" : void 0), z(e._zod, "optout", () => t.options.some((n) => n._zod.optout === "optional") ? "optional" : void 0), z(e._zod, "values", () => {
    if (t.options.every((n) => n._zod.values))
      return new Set(t.options.flatMap((n) => Array.from(n._zod.values)));
  }), z(e._zod, "pattern", () => {
    if (t.options.every((n) => n._zod.pattern)) {
      const n = t.options.map((o) => o._zod.pattern);
      return new RegExp(`^(${n.map((o) => Ke(o.source)).join("|")})$`);
    }
  }), e._zod.parse = (n, o) => {
    let r = !1;
    const s = [];
    for (const i of t.options) {
      const c = i._zod.run({
        value: n.value,
        issues: []
      }, o);
      if (c instanceof Promise)
        s.push(c), r = !0;
      else {
        if (c.issues.length === 0)
          return c;
        s.push(c);
      }
    }
    return r ? Promise.all(s).then((i) => Et(i, n, e, o)) : Et(s, n, e, o);
  };
}), Mr = /* @__PURE__ */ l("$ZodDiscriminatedUnion", (e, t) => {
  pn.init(e, t);
  const n = e._zod.parse;
  z(e._zod, "propValues", () => {
    const r = {};
    for (const s of t.options) {
      const i = s._zod.propValues;
      if (!i || Object.keys(i).length === 0)
        throw new Error(`Invalid discriminated union option at index "${t.options.indexOf(s)}"`);
      for (const [c, p] of Object.entries(i)) {
        r[c] || (r[c] = /* @__PURE__ */ new Set());
        for (const d of p)
          r[c].add(d);
      }
    }
    return r;
  });
  const o = We(() => {
    const r = t.options, s = /* @__PURE__ */ new Map();
    for (const i of r) {
      const c = i._zod.propValues[t.discriminator];
      if (!c || c.size === 0)
        throw new Error(`Invalid discriminated union option at index "${t.options.indexOf(i)}"`);
      for (const p of c) {
        if (s.has(p))
          throw new Error(`Duplicate discriminator value "${String(p)}"`);
        s.set(p, i);
      }
    }
    return s;
  });
  e._zod.parse = (r, s) => {
    const i = r.value;
    if (!Ee(i))
      return r.issues.push({
        code: "invalid_type",
        expected: "object",
        input: i,
        inst: e
      }), r;
    const c = o.value.get(i?.[t.discriminator]);
    return c ? c._zod.run(r, s) : t.unionFallback ? n(r, s) : (r.issues.push({
      code: "invalid_union",
      errors: [],
      note: "No matching discriminator",
      input: i,
      path: [t.discriminator],
      inst: e
    }), r);
  };
}), Lr = /* @__PURE__ */ l("$ZodIntersection", (e, t) => {
  E.init(e, t), e._zod.parse = (n, o) => {
    const r = n.value, s = t.left._zod.run({ value: r, issues: [] }, o), i = t.right._zod.run({ value: r, issues: [] }, o);
    return s instanceof Promise || i instanceof Promise ? Promise.all([s, i]).then(([p, d]) => xt(n, p, d)) : xt(n, s, i);
  };
});
function Ve(e, t) {
  if (e === t)
    return { valid: !0, data: e };
  if (e instanceof Date && t instanceof Date && +e == +t)
    return { valid: !0, data: e };
  if (xe(e) && xe(t)) {
    const n = Object.keys(t), o = Object.keys(e).filter((s) => n.indexOf(s) !== -1), r = { ...e, ...t };
    for (const s of o) {
      const i = Ve(e[s], t[s]);
      if (!i.valid)
        return {
          valid: !1,
          mergeErrorPath: [s, ...i.mergeErrorPath]
        };
      r[s] = i.data;
    }
    return { valid: !0, data: r };
  }
  if (Array.isArray(e) && Array.isArray(t)) {
    if (e.length !== t.length)
      return { valid: !1, mergeErrorPath: [] };
    const n = [];
    for (let o = 0; o < e.length; o++) {
      const r = e[o], s = t[o], i = Ve(r, s);
      if (!i.valid)
        return {
          valid: !1,
          mergeErrorPath: [o, ...i.mergeErrorPath]
        };
      n.push(i.data);
    }
    return { valid: !0, data: n };
  }
  return { valid: !1, mergeErrorPath: [] };
}
function xt(e, t, n) {
  if (t.issues.length && e.issues.push(...t.issues), n.issues.length && e.issues.push(...n.issues), le(e))
    return e;
  const o = Ve(t.value, n.value);
  if (!o.valid)
    throw new Error(`Unmergable intersection. Error path: ${JSON.stringify(o.mergeErrorPath)}`);
  return e.value = o.data, e;
}
const Ur = /* @__PURE__ */ l("$ZodRecord", (e, t) => {
  E.init(e, t), e._zod.parse = (n, o) => {
    const r = n.value;
    if (!xe(r))
      return n.issues.push({
        expected: "record",
        code: "invalid_type",
        input: r,
        inst: e
      }), n;
    const s = [];
    if (t.keyType._zod.values) {
      const i = t.keyType._zod.values;
      n.value = {};
      for (const p of i)
        if (typeof p == "string" || typeof p == "number" || typeof p == "symbol") {
          const d = t.valueType._zod.run({ value: r[p], issues: [] }, o);
          d instanceof Promise ? s.push(d.then((f) => {
            f.issues.length && n.issues.push(...ee(p, f.issues)), n.value[p] = f.value;
          })) : (d.issues.length && n.issues.push(...ee(p, d.issues)), n.value[p] = d.value);
        }
      let c;
      for (const p in r)
        i.has(p) || (c = c ?? [], c.push(p));
      c && c.length > 0 && n.issues.push({
        code: "unrecognized_keys",
        input: r,
        inst: e,
        keys: c
      });
    } else {
      n.value = {};
      for (const i of Reflect.ownKeys(r)) {
        if (i === "__proto__")
          continue;
        const c = t.keyType._zod.run({ value: i, issues: [] }, o);
        if (c instanceof Promise)
          throw new Error("Async schemas not supported in object keys currently");
        if (c.issues.length) {
          n.issues.push({
            origin: "record",
            code: "invalid_key",
            issues: c.issues.map((d) => Q(d, o, W())),
            input: i,
            path: [i],
            inst: e
          }), n.value[c.value] = c.value;
          continue;
        }
        const p = t.valueType._zod.run({ value: r[i], issues: [] }, o);
        p instanceof Promise ? s.push(p.then((d) => {
          d.issues.length && n.issues.push(...ee(i, d.issues)), n.value[c.value] = d.value;
        })) : (p.issues.length && n.issues.push(...ee(i, p.issues)), n.value[c.value] = p.value);
      }
    }
    return s.length ? Promise.all(s).then(() => n) : n;
  };
}), Dr = /* @__PURE__ */ l("$ZodEnum", (e, t) => {
  E.init(e, t);
  const n = eo(t.entries);
  e._zod.values = new Set(n), e._zod.pattern = new RegExp(`^(${n.filter((o) => ro.has(typeof o)).map((o) => typeof o == "string" ? he(o) : o.toString()).join("|")})$`), e._zod.parse = (o, r) => {
    const s = o.value;
    return e._zod.values.has(s) || o.issues.push({
      code: "invalid_value",
      values: n,
      input: s,
      inst: e
    }), o;
  };
}), Fr = /* @__PURE__ */ l("$ZodLiteral", (e, t) => {
  E.init(e, t), e._zod.values = new Set(t.values), e._zod.pattern = new RegExp(`^(${t.values.map((n) => typeof n == "string" ? he(n) : n ? n.toString() : String(n)).join("|")})$`), e._zod.parse = (n, o) => {
    const r = n.value;
    return e._zod.values.has(r) || n.issues.push({
      code: "invalid_value",
      values: t.values,
      input: r,
      inst: e
    }), n;
  };
}), Hr = /* @__PURE__ */ l("$ZodTransform", (e, t) => {
  E.init(e, t), e._zod.parse = (n, o) => {
    const r = t.transform(n.value, n);
    if (o.async)
      return (r instanceof Promise ? r : Promise.resolve(r)).then((i) => (n.value = i, n));
    if (r instanceof Promise)
      throw new de();
    return n.value = r, n;
  };
}), Gr = /* @__PURE__ */ l("$ZodOptional", (e, t) => {
  E.init(e, t), e._zod.optin = "optional", e._zod.optout = "optional", z(e._zod, "values", () => t.innerType._zod.values ? /* @__PURE__ */ new Set([...t.innerType._zod.values, void 0]) : void 0), z(e._zod, "pattern", () => {
    const n = t.innerType._zod.pattern;
    return n ? new RegExp(`^(${Ke(n.source)})?$`) : void 0;
  }), e._zod.parse = (n, o) => t.innerType._zod.optin === "optional" ? t.innerType._zod.run(n, o) : n.value === void 0 ? n : t.innerType._zod.run(n, o);
}), Vr = /* @__PURE__ */ l("$ZodNullable", (e, t) => {
  E.init(e, t), z(e._zod, "optin", () => t.innerType._zod.optin), z(e._zod, "optout", () => t.innerType._zod.optout), z(e._zod, "pattern", () => {
    const n = t.innerType._zod.pattern;
    return n ? new RegExp(`^(${Ke(n.source)}|null)$`) : void 0;
  }), z(e._zod, "values", () => t.innerType._zod.values ? /* @__PURE__ */ new Set([...t.innerType._zod.values, null]) : void 0), e._zod.parse = (n, o) => n.value === null ? n : t.innerType._zod.run(n, o);
}), Jr = /* @__PURE__ */ l("$ZodDefault", (e, t) => {
  E.init(e, t), e._zod.optin = "optional", z(e._zod, "values", () => t.innerType._zod.values), e._zod.parse = (n, o) => {
    if (n.value === void 0)
      return n.value = t.defaultValue, n;
    const r = t.innerType._zod.run(n, o);
    return r instanceof Promise ? r.then((s) => Zt(s, t)) : Zt(r, t);
  };
});
function Zt(e, t) {
  return e.value === void 0 && (e.value = t.defaultValue), e;
}
const Br = /* @__PURE__ */ l("$ZodPrefault", (e, t) => {
  E.init(e, t), e._zod.optin = "optional", z(e._zod, "values", () => t.innerType._zod.values), e._zod.parse = (n, o) => (n.value === void 0 && (n.value = t.defaultValue), t.innerType._zod.run(n, o));
}), Wr = /* @__PURE__ */ l("$ZodNonOptional", (e, t) => {
  E.init(e, t), z(e._zod, "values", () => {
    const n = t.innerType._zod.values;
    return n ? new Set([...n].filter((o) => o !== void 0)) : void 0;
  }), e._zod.parse = (n, o) => {
    const r = t.innerType._zod.run(n, o);
    return r instanceof Promise ? r.then((s) => Pt(s, e)) : Pt(r, e);
  };
});
function Pt(e, t) {
  return !e.issues.length && e.value === void 0 && e.issues.push({
    code: "invalid_type",
    expected: "nonoptional",
    input: e.value,
    inst: t
  }), e;
}
const Qr = /* @__PURE__ */ l("$ZodCatch", (e, t) => {
  E.init(e, t), e._zod.optin = "optional", z(e._zod, "optout", () => t.innerType._zod.optout), z(e._zod, "values", () => t.innerType._zod.values), e._zod.parse = (n, o) => {
    const r = t.innerType._zod.run(n, o);
    return r instanceof Promise ? r.then((s) => (n.value = s.value, s.issues.length && (n.value = t.catchValue({
      ...n,
      error: {
        issues: s.issues.map((i) => Q(i, o, W()))
      },
      input: n.value
    }), n.issues = []), n)) : (n.value = r.value, r.issues.length && (n.value = t.catchValue({
      ...n,
      error: {
        issues: r.issues.map((s) => Q(s, o, W()))
      },
      input: n.value
    }), n.issues = []), n);
  };
}), Kr = /* @__PURE__ */ l("$ZodPipe", (e, t) => {
  E.init(e, t), z(e._zod, "values", () => t.in._zod.values), z(e._zod, "optin", () => t.in._zod.optin), z(e._zod, "optout", () => t.out._zod.optout), e._zod.parse = (n, o) => {
    const r = t.in._zod.run(n, o);
    return r instanceof Promise ? r.then((s) => qt(s, t, o)) : qt(r, t, o);
  };
});
function qt(e, t, n) {
  return le(e) ? e : t.out._zod.run({ value: e.value, issues: e.issues }, n);
}
const Yr = /* @__PURE__ */ l("$ZodReadonly", (e, t) => {
  E.init(e, t), z(e._zod, "propValues", () => t.innerType._zod.propValues), z(e._zod, "values", () => t.innerType._zod.values), z(e._zod, "optin", () => t.innerType._zod.optin), z(e._zod, "optout", () => t.innerType._zod.optout), e._zod.parse = (n, o) => {
    const r = t.innerType._zod.run(n, o);
    return r instanceof Promise ? r.then(Ct) : Ct(r);
  };
});
function Ct(e) {
  return e.value = Object.freeze(e.value), e;
}
const Xr = /* @__PURE__ */ l("$ZodCustom", (e, t) => {
  F.init(e, t), E.init(e, t), e._zod.parse = (n, o) => n, e._zod.check = (n) => {
    const o = n.value, r = t.fn(o);
    if (r instanceof Promise)
      return r.then((s) => jt(s, n, o, e));
    jt(r, n, o, e);
  };
});
function jt(e, t, n, o) {
  if (!e) {
    const r = {
      code: "custom",
      input: n,
      inst: o,
      // incorporates params.error into issue reporting
      path: [...o._zod.def.path ?? []],
      // incorporates params.error into issue reporting
      continue: !o._zod.def.abort
      // params: inst._zod.def.params,
    };
    o._zod.def.params && (r.params = o._zod.def.params), t.issues.push(me(r));
  }
}
class es {
  constructor() {
    this._map = /* @__PURE__ */ new Map(), this._idmap = /* @__PURE__ */ new Map();
  }
  add(t, ...n) {
    const o = n[0];
    if (this._map.set(t, o), o && typeof o == "object" && "id" in o) {
      if (this._idmap.has(o.id))
        throw new Error(`ID ${o.id} already exists in the registry`);
      this._idmap.set(o.id, t);
    }
    return this;
  }
  clear() {
    return this._map = /* @__PURE__ */ new Map(), this._idmap = /* @__PURE__ */ new Map(), this;
  }
  remove(t) {
    const n = this._map.get(t);
    return n && typeof n == "object" && "id" in n && this._idmap.delete(n.id), this._map.delete(t), this;
  }
  get(t) {
    const n = t._zod.parent;
    if (n) {
      const o = { ...this.get(n) ?? {} };
      return delete o.id, { ...o, ...this._map.get(t) };
    }
    return this._map.get(t);
  }
  has(t) {
    return this._map.has(t);
  }
}
function ts() {
  return new es();
}
const $e = /* @__PURE__ */ ts();
function ns(e, t) {
  return new e({
    type: "string",
    ..._(t)
  });
}
function os(e, t) {
  return new e({
    type: "string",
    format: "email",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function Nt(e, t) {
  return new e({
    type: "string",
    format: "guid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function rs(e, t) {
  return new e({
    type: "string",
    format: "uuid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ss(e, t) {
  return new e({
    type: "string",
    format: "uuid",
    check: "string_format",
    abort: !1,
    version: "v4",
    ..._(t)
  });
}
function is(e, t) {
  return new e({
    type: "string",
    format: "uuid",
    check: "string_format",
    abort: !1,
    version: "v6",
    ..._(t)
  });
}
function as(e, t) {
  return new e({
    type: "string",
    format: "uuid",
    check: "string_format",
    abort: !1,
    version: "v7",
    ..._(t)
  });
}
function cs(e, t) {
  return new e({
    type: "string",
    format: "url",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function us(e, t) {
  return new e({
    type: "string",
    format: "emoji",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ls(e, t) {
  return new e({
    type: "string",
    format: "nanoid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ps(e, t) {
  return new e({
    type: "string",
    format: "cuid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ds(e, t) {
  return new e({
    type: "string",
    format: "cuid2",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ms(e, t) {
  return new e({
    type: "string",
    format: "ulid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function hs(e, t) {
  return new e({
    type: "string",
    format: "xid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function fs(e, t) {
  return new e({
    type: "string",
    format: "ksuid",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function gs(e, t) {
  return new e({
    type: "string",
    format: "ipv4",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function _s(e, t) {
  return new e({
    type: "string",
    format: "ipv6",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function vs(e, t) {
  return new e({
    type: "string",
    format: "cidrv4",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function bs(e, t) {
  return new e({
    type: "string",
    format: "cidrv6",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ws(e, t) {
  return new e({
    type: "string",
    format: "base64",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ks(e, t) {
  return new e({
    type: "string",
    format: "base64url",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function ys(e, t) {
  return new e({
    type: "string",
    format: "e164",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function Ss(e, t) {
  return new e({
    type: "string",
    format: "jwt",
    check: "string_format",
    abort: !1,
    ..._(t)
  });
}
function Ts(e, t) {
  return new e({
    type: "string",
    format: "datetime",
    check: "string_format",
    offset: !1,
    local: !1,
    precision: null,
    ..._(t)
  });
}
function $s(e, t) {
  return new e({
    type: "string",
    format: "date",
    check: "string_format",
    ..._(t)
  });
}
function Rs(e, t) {
  return new e({
    type: "string",
    format: "time",
    check: "string_format",
    precision: null,
    ..._(t)
  });
}
function zs(e, t) {
  return new e({
    type: "string",
    format: "duration",
    check: "string_format",
    ..._(t)
  });
}
function Is(e, t) {
  return new e({
    type: "number",
    checks: [],
    ..._(t)
  });
}
function Es(e, t) {
  return new e({
    type: "number",
    check: "number_format",
    abort: !1,
    format: "safeint",
    ..._(t)
  });
}
function xs(e, t) {
  return new e({
    type: "boolean",
    ..._(t)
  });
}
function Zs(e, t) {
  return new e({
    type: "null",
    ..._(t)
  });
}
function Ps(e) {
  return new e({
    type: "unknown"
  });
}
function qs(e, t) {
  return new e({
    type: "never",
    ..._(t)
  });
}
function Ot(e, t) {
  return new an({
    check: "less_than",
    ..._(t),
    value: e,
    inclusive: !1
  });
}
function Fe(e, t) {
  return new an({
    check: "less_than",
    ..._(t),
    value: e,
    inclusive: !0
  });
}
function At(e, t) {
  return new cn({
    check: "greater_than",
    ..._(t),
    value: e,
    inclusive: !1
  });
}
function He(e, t) {
  return new cn({
    check: "greater_than",
    ..._(t),
    value: e,
    inclusive: !0
  });
}
function Mt(e, t) {
  return new Jo({
    check: "multiple_of",
    ..._(t),
    value: e
  });
}
function dn(e, t) {
  return new Wo({
    check: "max_length",
    ..._(t),
    maximum: e
  });
}
function Ze(e, t) {
  return new Qo({
    check: "min_length",
    ..._(t),
    minimum: e
  });
}
function mn(e, t) {
  return new Ko({
    check: "length_equals",
    ..._(t),
    length: e
  });
}
function Cs(e, t) {
  return new Yo({
    check: "string_format",
    format: "regex",
    ..._(t),
    pattern: e
  });
}
function js(e) {
  return new Xo({
    check: "string_format",
    format: "lowercase",
    ..._(e)
  });
}
function Ns(e) {
  return new er({
    check: "string_format",
    format: "uppercase",
    ..._(e)
  });
}
function Os(e, t) {
  return new tr({
    check: "string_format",
    format: "includes",
    ..._(t),
    includes: e
  });
}
function As(e, t) {
  return new nr({
    check: "string_format",
    format: "starts_with",
    ..._(t),
    prefix: e
  });
}
function Ms(e, t) {
  return new or({
    check: "string_format",
    format: "ends_with",
    ..._(t),
    suffix: e
  });
}
function fe(e) {
  return new rr({
    check: "overwrite",
    tx: e
  });
}
function Ls(e) {
  return fe((t) => t.normalize(e));
}
function Us() {
  return fe((e) => e.trim());
}
function Ds() {
  return fe((e) => e.toLowerCase());
}
function Fs() {
  return fe((e) => e.toUpperCase());
}
function Hs(e, t, n) {
  return new e({
    type: "array",
    element: t,
    // get element() {
    //   return element;
    // },
    ..._(n)
  });
}
function Gs(e, t, n) {
  const o = _(n);
  return o.abort ?? (o.abort = !0), new e({
    type: "custom",
    check: "custom",
    fn: t,
    ...o
  });
}
function Vs(e, t, n) {
  return new e({
    type: "custom",
    check: "custom",
    fn: t,
    ..._(n)
  });
}
function Ae(e) {
  return !!e._zod;
}
function pe(e, t) {
  return Ae(e) ? en(e, t) : e.safeParse(t);
}
function hn(e) {
  if (!e)
    return;
  let t;
  if (Ae(e) ? t = e._zod?.def?.shape : t = e.shape, !!t) {
    if (typeof t == "function")
      try {
        return t();
      } catch {
        return;
      }
    return t;
  }
}
function Js(e) {
  if (Ae(e)) {
    const s = e._zod?.def;
    if (s) {
      if (s.value !== void 0)
        return s.value;
      if (Array.isArray(s.values) && s.values.length > 0)
        return s.values[0];
    }
  }
  const n = e._def;
  if (n) {
    if (n.value !== void 0)
      return n.value;
    if (Array.isArray(n.values) && n.values.length > 0)
      return n.values[0];
  }
  const o = e.value;
  if (o !== void 0)
    return o;
}
const Bs = /* @__PURE__ */ l("ZodISODateTime", (e, t) => {
  vr.init(e, t), N.init(e, t);
});
function fn(e) {
  return Ts(Bs, e);
}
const Ws = /* @__PURE__ */ l("ZodISODate", (e, t) => {
  br.init(e, t), N.init(e, t);
});
function Qs(e) {
  return $s(Ws, e);
}
const Ks = /* @__PURE__ */ l("ZodISOTime", (e, t) => {
  wr.init(e, t), N.init(e, t);
});
function Ys(e) {
  return Rs(Ks, e);
}
const Xs = /* @__PURE__ */ l("ZodISODuration", (e, t) => {
  kr.init(e, t), N.init(e, t);
});
function ei(e) {
  return zs(Xs, e);
}
const ti = (e, t) => {
  Kt.init(e, t), e.name = "ZodError", Object.defineProperties(e, {
    format: {
      value: (n) => fo(e, n)
      // enumerable: false,
    },
    flatten: {
      value: (n) => ho(e, n)
      // enumerable: false,
    },
    addIssue: {
      value: (n) => e.issues.push(n)
      // enumerable: false,
    },
    addIssues: {
      value: (n) => e.issues.push(...n)
      // enumerable: false,
    },
    isEmpty: {
      get() {
        return e.issues.length === 0;
      }
      // enumerable: false,
    }
  });
}, Me = l("ZodError", ti, {
  Parent: Error
}), ni = /* @__PURE__ */ go(Me), oi = /* @__PURE__ */ _o(Me), ri = /* @__PURE__ */ Xt(Me), si = /* @__PURE__ */ tn(Me), j = /* @__PURE__ */ l("ZodType", (e, t) => (E.init(e, t), e.def = t, Object.defineProperty(e, "_def", { value: t }), e.check = (...n) => e.clone(
  {
    ...t,
    checks: [
      ...t.checks ?? [],
      ...n.map((o) => typeof o == "function" ? { _zod: { check: o, def: { check: "custom" }, onattach: [] } } : o)
    ]
  }
  // { parent: true }
), e.clone = (n, o) => te(e, n, o), e.brand = () => e, e.register = (n, o) => (n.add(e, o), e), e.parse = (n, o) => ni(e, n, o, { callee: e.parse }), e.safeParse = (n, o) => ri(e, n, o), e.parseAsync = async (n, o) => oi(e, n, o, { callee: e.parseAsync }), e.safeParseAsync = async (n, o) => si(e, n, o), e.spa = e.safeParseAsync, e.refine = (n, o) => e.check(Qi(n, o)), e.superRefine = (n) => e.check(Ki(n)), e.overwrite = (n) => e.check(fe(n)), e.optional = () => q(e), e.nullable = () => Dt(e), e.nullish = () => q(Dt(e)), e.nonoptional = (n) => Di(e, n), e.array = () => T(e), e.or = (n) => P([e, n]), e.and = (n) => et(e, n), e.transform = (n) => Be(e, kn(n)), e.default = (n) => Mi(e, n), e.prefault = (n) => Ui(e, n), e.catch = (n) => Hi(e, n), e.pipe = (n) => Be(e, n), e.readonly = () => Ji(e), e.describe = (n) => {
  const o = e.clone();
  return $e.add(o, { description: n }), o;
}, Object.defineProperty(e, "description", {
  get() {
    return $e.get(e)?.description;
  },
  configurable: !0
}), e.meta = (...n) => {
  if (n.length === 0)
    return $e.get(e);
  const o = e.clone();
  return $e.add(o, n[0]), o;
}, e.isOptional = () => e.safeParse(void 0).success, e.isNullable = () => e.safeParse(null).success, e)), gn = /* @__PURE__ */ l("_ZodString", (e, t) => {
  Xe.init(e, t), j.init(e, t);
  const n = e._zod.bag;
  e.format = n.format ?? null, e.minLength = n.minimum ?? null, e.maxLength = n.maximum ?? null, e.regex = (...o) => e.check(Cs(...o)), e.includes = (...o) => e.check(Os(...o)), e.startsWith = (...o) => e.check(As(...o)), e.endsWith = (...o) => e.check(Ms(...o)), e.min = (...o) => e.check(Ze(...o)), e.max = (...o) => e.check(dn(...o)), e.length = (...o) => e.check(mn(...o)), e.nonempty = (...o) => e.check(Ze(1, ...o)), e.lowercase = (o) => e.check(js(o)), e.uppercase = (o) => e.check(Ns(o)), e.trim = () => e.check(Us()), e.normalize = (...o) => e.check(Ls(...o)), e.toLowerCase = () => e.check(Ds()), e.toUpperCase = () => e.check(Fs());
}), ii = /* @__PURE__ */ l("ZodString", (e, t) => {
  Xe.init(e, t), gn.init(e, t), e.email = (n) => e.check(os(ai, n)), e.url = (n) => e.check(cs(ci, n)), e.jwt = (n) => e.check(Ss(Si, n)), e.emoji = (n) => e.check(us(ui, n)), e.guid = (n) => e.check(Nt(Lt, n)), e.uuid = (n) => e.check(rs(Re, n)), e.uuidv4 = (n) => e.check(ss(Re, n)), e.uuidv6 = (n) => e.check(is(Re, n)), e.uuidv7 = (n) => e.check(as(Re, n)), e.nanoid = (n) => e.check(ls(li, n)), e.guid = (n) => e.check(Nt(Lt, n)), e.cuid = (n) => e.check(ps(pi, n)), e.cuid2 = (n) => e.check(ds(di, n)), e.ulid = (n) => e.check(ms(mi, n)), e.base64 = (n) => e.check(ws(wi, n)), e.base64url = (n) => e.check(ks(ki, n)), e.xid = (n) => e.check(hs(hi, n)), e.ksuid = (n) => e.check(fs(fi, n)), e.ipv4 = (n) => e.check(gs(gi, n)), e.ipv6 = (n) => e.check(_s(_i, n)), e.cidrv4 = (n) => e.check(vs(vi, n)), e.cidrv6 = (n) => e.check(bs(bi, n)), e.e164 = (n) => e.check(ys(yi, n)), e.datetime = (n) => e.check(fn(n)), e.date = (n) => e.check(Qs(n)), e.time = (n) => e.check(Ys(n)), e.duration = (n) => e.check(ei(n));
});
function u(e) {
  return ns(ii, e);
}
const N = /* @__PURE__ */ l("ZodStringFormat", (e, t) => {
  Z.init(e, t), gn.init(e, t);
}), ai = /* @__PURE__ */ l("ZodEmail", (e, t) => {
  ur.init(e, t), N.init(e, t);
}), Lt = /* @__PURE__ */ l("ZodGUID", (e, t) => {
  ar.init(e, t), N.init(e, t);
}), Re = /* @__PURE__ */ l("ZodUUID", (e, t) => {
  cr.init(e, t), N.init(e, t);
}), ci = /* @__PURE__ */ l("ZodURL", (e, t) => {
  lr.init(e, t), N.init(e, t);
}), ui = /* @__PURE__ */ l("ZodEmoji", (e, t) => {
  pr.init(e, t), N.init(e, t);
}), li = /* @__PURE__ */ l("ZodNanoID", (e, t) => {
  dr.init(e, t), N.init(e, t);
}), pi = /* @__PURE__ */ l("ZodCUID", (e, t) => {
  mr.init(e, t), N.init(e, t);
}), di = /* @__PURE__ */ l("ZodCUID2", (e, t) => {
  hr.init(e, t), N.init(e, t);
}), mi = /* @__PURE__ */ l("ZodULID", (e, t) => {
  fr.init(e, t), N.init(e, t);
}), hi = /* @__PURE__ */ l("ZodXID", (e, t) => {
  gr.init(e, t), N.init(e, t);
}), fi = /* @__PURE__ */ l("ZodKSUID", (e, t) => {
  _r.init(e, t), N.init(e, t);
}), gi = /* @__PURE__ */ l("ZodIPv4", (e, t) => {
  yr.init(e, t), N.init(e, t);
}), _i = /* @__PURE__ */ l("ZodIPv6", (e, t) => {
  Sr.init(e, t), N.init(e, t);
}), vi = /* @__PURE__ */ l("ZodCIDRv4", (e, t) => {
  Tr.init(e, t), N.init(e, t);
}), bi = /* @__PURE__ */ l("ZodCIDRv6", (e, t) => {
  $r.init(e, t), N.init(e, t);
}), wi = /* @__PURE__ */ l("ZodBase64", (e, t) => {
  Rr.init(e, t), N.init(e, t);
}), ki = /* @__PURE__ */ l("ZodBase64URL", (e, t) => {
  Ir.init(e, t), N.init(e, t);
}), yi = /* @__PURE__ */ l("ZodE164", (e, t) => {
  Er.init(e, t), N.init(e, t);
}), Si = /* @__PURE__ */ l("ZodJWT", (e, t) => {
  Zr.init(e, t), N.init(e, t);
}), _n = /* @__PURE__ */ l("ZodNumber", (e, t) => {
  ln.init(e, t), j.init(e, t), e.gt = (o, r) => e.check(At(o, r)), e.gte = (o, r) => e.check(He(o, r)), e.min = (o, r) => e.check(He(o, r)), e.lt = (o, r) => e.check(Ot(o, r)), e.lte = (o, r) => e.check(Fe(o, r)), e.max = (o, r) => e.check(Fe(o, r)), e.int = (o) => e.check(Ut(o)), e.safe = (o) => e.check(Ut(o)), e.positive = (o) => e.check(At(0, o)), e.nonnegative = (o) => e.check(He(0, o)), e.negative = (o) => e.check(Ot(0, o)), e.nonpositive = (o) => e.check(Fe(0, o)), e.multipleOf = (o, r) => e.check(Mt(o, r)), e.step = (o, r) => e.check(Mt(o, r)), e.finite = () => e;
  const n = e._zod.bag;
  e.minValue = Math.max(n.minimum ?? Number.NEGATIVE_INFINITY, n.exclusiveMinimum ?? Number.NEGATIVE_INFINITY) ?? null, e.maxValue = Math.min(n.maximum ?? Number.POSITIVE_INFINITY, n.exclusiveMaximum ?? Number.POSITIVE_INFINITY) ?? null, e.isInt = (n.format ?? "").includes("int") || Number.isSafeInteger(n.multipleOf ?? 0.5), e.isFinite = !0, e.format = n.format ?? null;
});
function $(e) {
  return Is(_n, e);
}
const Ti = /* @__PURE__ */ l("ZodNumberFormat", (e, t) => {
  Pr.init(e, t), _n.init(e, t);
});
function Ut(e) {
  return Es(Ti, e);
}
const $i = /* @__PURE__ */ l("ZodBoolean", (e, t) => {
  qr.init(e, t), j.init(e, t);
});
function A(e) {
  return xs($i, e);
}
const Ri = /* @__PURE__ */ l("ZodNull", (e, t) => {
  Cr.init(e, t), j.init(e, t);
});
function zi(e) {
  return Zs(Ri, e);
}
const Ii = /* @__PURE__ */ l("ZodUnknown", (e, t) => {
  jr.init(e, t), j.init(e, t);
});
function C() {
  return Ps(Ii);
}
const Ei = /* @__PURE__ */ l("ZodNever", (e, t) => {
  Nr.init(e, t), j.init(e, t);
});
function xi(e) {
  return qs(Ei, e);
}
const Zi = /* @__PURE__ */ l("ZodArray", (e, t) => {
  Or.init(e, t), j.init(e, t), e.element = t.element, e.min = (n, o) => e.check(Ze(n, o)), e.nonempty = (n) => e.check(Ze(1, n)), e.max = (n, o) => e.check(dn(n, o)), e.length = (n, o) => e.check(mn(n, o)), e.unwrap = () => e.element;
});
function T(e, t) {
  return Hs(Zi, e, t);
}
const vn = /* @__PURE__ */ l("ZodObject", (e, t) => {
  Ar.init(e, t), j.init(e, t), z(e, "shape", () => t.shape), e.keyof = () => H(Object.keys(e._zod.def.shape)), e.catchall = (n) => e.clone({ ...e._zod.def, catchall: n }), e.passthrough = () => e.clone({ ...e._zod.def, catchall: C() }), e.loose = () => e.clone({ ...e._zod.def, catchall: C() }), e.strict = () => e.clone({ ...e._zod.def, catchall: xi() }), e.strip = () => e.clone({ ...e._zod.def, catchall: void 0 }), e.extend = (n) => uo(e, n), e.merge = (n) => lo(e, n), e.pick = (n) => ao(e, n), e.omit = (n) => co(e, n), e.partial = (...n) => po(yn, e, n[0]), e.required = (...n) => mo(Sn, e, n[0]);
});
function h(e, t) {
  const n = {
    type: "object",
    get shape() {
      return Ne(this, "shape", { ...e }), this.shape;
    },
    ..._(t)
  };
  return new vn(n);
}
function D(e, t) {
  return new vn({
    type: "object",
    get shape() {
      return Ne(this, "shape", { ...e }), this.shape;
    },
    catchall: C(),
    ..._(t)
  });
}
const bn = /* @__PURE__ */ l("ZodUnion", (e, t) => {
  pn.init(e, t), j.init(e, t), e.options = t.options;
});
function P(e, t) {
  return new bn({
    type: "union",
    options: e,
    ..._(t)
  });
}
const Pi = /* @__PURE__ */ l("ZodDiscriminatedUnion", (e, t) => {
  bn.init(e, t), Mr.init(e, t);
});
function wn(e, t, n) {
  return new Pi({
    type: "union",
    options: t,
    discriminator: e,
    ..._(n)
  });
}
const qi = /* @__PURE__ */ l("ZodIntersection", (e, t) => {
  Lr.init(e, t), j.init(e, t);
});
function et(e, t) {
  return new qi({
    type: "intersection",
    left: e,
    right: t
  });
}
const Ci = /* @__PURE__ */ l("ZodRecord", (e, t) => {
  Ur.init(e, t), j.init(e, t), e.keyType = t.keyType, e.valueType = t.valueType;
});
function I(e, t, n) {
  return new Ci({
    type: "record",
    keyType: e,
    valueType: t,
    ..._(n)
  });
}
const Je = /* @__PURE__ */ l("ZodEnum", (e, t) => {
  Dr.init(e, t), j.init(e, t), e.enum = t.entries, e.options = Object.values(t.entries);
  const n = new Set(Object.keys(t.entries));
  e.extract = (o, r) => {
    const s = {};
    for (const i of o)
      if (n.has(i))
        s[i] = t.entries[i];
      else
        throw new Error(`Key ${i} not found in enum`);
    return new Je({
      ...t,
      checks: [],
      ..._(r),
      entries: s
    });
  }, e.exclude = (o, r) => {
    const s = { ...t.entries };
    for (const i of o)
      if (n.has(i))
        delete s[i];
      else
        throw new Error(`Key ${i} not found in enum`);
    return new Je({
      ...t,
      checks: [],
      ..._(r),
      entries: s
    });
  };
});
function H(e, t) {
  const n = Array.isArray(e) ? Object.fromEntries(e.map((o) => [o, o])) : e;
  return new Je({
    type: "enum",
    entries: n,
    ..._(t)
  });
}
const ji = /* @__PURE__ */ l("ZodLiteral", (e, t) => {
  Fr.init(e, t), j.init(e, t), e.values = new Set(t.values), Object.defineProperty(e, "value", {
    get() {
      if (t.values.length > 1)
        throw new Error("This schema contains multiple valid literal values. Use `.values` instead.");
      return t.values[0];
    }
  });
});
function v(e, t) {
  return new ji({
    type: "literal",
    values: Array.isArray(e) ? e : [e],
    ..._(t)
  });
}
const Ni = /* @__PURE__ */ l("ZodTransform", (e, t) => {
  Hr.init(e, t), j.init(e, t), e._zod.parse = (n, o) => {
    n.addIssue = (s) => {
      if (typeof s == "string")
        n.issues.push(me(s, n.value, t));
      else {
        const i = s;
        i.fatal && (i.continue = !1), i.code ?? (i.code = "custom"), i.input ?? (i.input = n.value), i.inst ?? (i.inst = e), i.continue ?? (i.continue = !0), n.issues.push(me(i));
      }
    };
    const r = t.transform(n.value, n);
    return r instanceof Promise ? r.then((s) => (n.value = s, n)) : (n.value = r, n);
  };
});
function kn(e) {
  return new Ni({
    type: "transform",
    transform: e
  });
}
const yn = /* @__PURE__ */ l("ZodOptional", (e, t) => {
  Gr.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType;
});
function q(e) {
  return new yn({
    type: "optional",
    innerType: e
  });
}
const Oi = /* @__PURE__ */ l("ZodNullable", (e, t) => {
  Vr.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType;
});
function Dt(e) {
  return new Oi({
    type: "nullable",
    innerType: e
  });
}
const Ai = /* @__PURE__ */ l("ZodDefault", (e, t) => {
  Jr.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType, e.removeDefault = e.unwrap;
});
function Mi(e, t) {
  return new Ai({
    type: "default",
    innerType: e,
    get defaultValue() {
      return typeof t == "function" ? t() : t;
    }
  });
}
const Li = /* @__PURE__ */ l("ZodPrefault", (e, t) => {
  Br.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType;
});
function Ui(e, t) {
  return new Li({
    type: "prefault",
    innerType: e,
    get defaultValue() {
      return typeof t == "function" ? t() : t;
    }
  });
}
const Sn = /* @__PURE__ */ l("ZodNonOptional", (e, t) => {
  Wr.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType;
});
function Di(e, t) {
  return new Sn({
    type: "nonoptional",
    innerType: e,
    ..._(t)
  });
}
const Fi = /* @__PURE__ */ l("ZodCatch", (e, t) => {
  Qr.init(e, t), j.init(e, t), e.unwrap = () => e._zod.def.innerType, e.removeCatch = e.unwrap;
});
function Hi(e, t) {
  return new Fi({
    type: "catch",
    innerType: e,
    catchValue: typeof t == "function" ? t : () => t
  });
}
const Gi = /* @__PURE__ */ l("ZodPipe", (e, t) => {
  Kr.init(e, t), j.init(e, t), e.in = t.in, e.out = t.out;
});
function Be(e, t) {
  return new Gi({
    type: "pipe",
    in: e,
    out: t
    // ...util.normalizeParams(params),
  });
}
const Vi = /* @__PURE__ */ l("ZodReadonly", (e, t) => {
  Yr.init(e, t), j.init(e, t);
});
function Ji(e) {
  return new Vi({
    type: "readonly",
    innerType: e
  });
}
const Tn = /* @__PURE__ */ l("ZodCustom", (e, t) => {
  Xr.init(e, t), j.init(e, t);
});
function Bi(e) {
  const t = new F({
    check: "custom"
    // ...util.normalizeParams(params),
  });
  return t._zod.check = e, t;
}
function Wi(e, t) {
  return Gs(Tn, e ?? (() => !0), t);
}
function Qi(e, t = {}) {
  return Vs(Tn, e, t);
}
function Ki(e) {
  const t = Bi((n) => (n.addIssue = (o) => {
    if (typeof o == "string")
      n.issues.push(me(o, n.value, t._zod.def));
    else {
      const r = o;
      r.fatal && (r.continue = !1), r.code ?? (r.code = "custom"), r.input ?? (r.input = n.value), r.inst ?? (r.inst = t), r.continue ?? (r.continue = !t._zod.def.abort), n.issues.push(me(r));
    }
  }, e(n.value, n)));
  return t;
}
function $n(e, t) {
  return Be(kn(e), t);
}
const Rn = "2025-11-25", Yi = [Rn, "2025-06-18", "2025-03-26", "2024-11-05", "2024-10-07"], X = "io.modelcontextprotocol/related-task", Le = "2.0", O = Wi((e) => e !== null && (typeof e == "object" || typeof e == "function")), zn = P([u(), $().int()]), In = u();
D({
  /**
   * Requested duration in milliseconds to retain task from creation.
   */
  ttl: $().optional(),
  /**
   * Time in milliseconds to wait between task status requests.
   */
  pollInterval: $().optional()
});
const Xi = h({
  ttl: $().optional()
}), ea = h({
  taskId: u()
}), tt = D({
  /**
   * If specified, the caller is requesting out-of-band progress notifications for this request (as represented by notifications/progress). The value of this parameter is an opaque token that will be attached to any subsequent notifications. The receiver is not obligated to provide these notifications.
   */
  progressToken: zn.optional(),
  /**
   * If specified, this request is related to the provided task.
   */
  [X]: ea.optional()
}), G = h({
  /**
   * See [General fields: `_meta`](/specification/draft/basic/index#meta) for notes on `_meta` usage.
   */
  _meta: tt.optional()
}), ge = G.extend({
  /**
   * If specified, the caller is requesting task-augmented execution for this request.
   * The request will return a CreateTaskResult immediately, and the actual result can be
   * retrieved later via tasks/result.
   *
   * Task augmentation is subject to capability negotiation - receivers MUST declare support
   * for task augmentation of specific request types in their capabilities.
   */
  task: Xi.optional()
}), ta = (e) => ge.safeParse(e).success, M = h({
  method: u(),
  params: G.loose().optional()
}), V = h({
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: tt.optional()
}), J = h({
  method: u(),
  params: V.loose().optional()
}), L = D({
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: tt.optional()
}), Ue = P([u(), $().int()]), En = h({
  jsonrpc: v(Le),
  id: Ue,
  ...M.shape
}).strict(), Ft = (e) => En.safeParse(e).success, xn = h({
  jsonrpc: v(Le),
  ...J.shape
}).strict(), na = (e) => xn.safeParse(e).success, nt = h({
  jsonrpc: v(Le),
  id: Ue,
  result: L
}).strict(), ze = (e) => nt.safeParse(e).success;
var S;
(function(e) {
  e[e.ConnectionClosed = -32e3] = "ConnectionClosed", e[e.RequestTimeout = -32001] = "RequestTimeout", e[e.ParseError = -32700] = "ParseError", e[e.InvalidRequest = -32600] = "InvalidRequest", e[e.MethodNotFound = -32601] = "MethodNotFound", e[e.InvalidParams = -32602] = "InvalidParams", e[e.InternalError = -32603] = "InternalError", e[e.UrlElicitationRequired = -32042] = "UrlElicitationRequired";
})(S || (S = {}));
const ot = h({
  jsonrpc: v(Le),
  id: Ue.optional(),
  error: h({
    /**
     * The error type that occurred.
     */
    code: $().int(),
    /**
     * A short description of the error. The message SHOULD be limited to a concise single sentence.
     */
    message: u(),
    /**
     * Additional information about the error. The value of this member is defined by the sender (e.g. detailed error information, nested errors etc.).
     */
    data: C().optional()
  })
}).strict(), oa = (e) => ot.safeParse(e).success, ra = P([
  En,
  xn,
  nt,
  ot
]);
P([nt, ot]);
const rt = L.strict(), sa = V.extend({
  /**
   * The ID of the request to cancel.
   *
   * This MUST correspond to the ID of a request previously issued in the same direction.
   */
  requestId: Ue.optional(),
  /**
   * An optional string describing the reason for the cancellation. This MAY be logged or presented to the user.
   */
  reason: u().optional()
}), st = J.extend({
  method: v("notifications/cancelled"),
  params: sa
}), ia = h({
  /**
   * URL or data URI for the icon.
   */
  src: u(),
  /**
   * Optional MIME type for the icon.
   */
  mimeType: u().optional(),
  /**
   * Optional array of strings that specify sizes at which the icon can be used.
   * Each string should be in WxH format (e.g., `"48x48"`, `"96x96"`) or `"any"` for scalable formats like SVG.
   *
   * If not provided, the client should assume that the icon can be used at any size.
   */
  sizes: T(u()).optional(),
  /**
   * Optional specifier for the theme this icon is designed for. `light` indicates
   * the icon is designed to be used with a light background, and `dark` indicates
   * the icon is designed to be used with a dark background.
   *
   * If not provided, the client should assume the icon can be used with any theme.
   */
  theme: H(["light", "dark"]).optional()
}), _e = h({
  /**
   * Optional set of sized icons that the client can display in a user interface.
   *
   * Clients that support rendering icons MUST support at least the following MIME types:
   * - `image/png` - PNG images (safe, universal compatibility)
   * - `image/jpeg` (and `image/jpg`) - JPEG images (safe, universal compatibility)
   *
   * Clients that support rendering icons SHOULD also support:
   * - `image/svg+xml` - SVG images (scalable but requires security precautions)
   * - `image/webp` - WebP images (modern, efficient format)
   */
  icons: T(ia).optional()
}), re = h({
  /** Intended for programmatic or logical use, but used as a display name in past specs or fallback */
  name: u(),
  /**
   * Intended for UI and end-user contexts — optimized to be human-readable and easily understood,
   * even by those unfamiliar with domain-specific terminology.
   *
   * If not provided, the name should be used for display (except for Tool,
   * where `annotations.title` should be given precedence over using `name`,
   * if present).
   */
  title: u().optional()
}), Zn = re.extend({
  ...re.shape,
  ..._e.shape,
  version: u(),
  /**
   * An optional URL of the website for this implementation.
   */
  websiteUrl: u().optional(),
  /**
   * An optional human-readable description of what this implementation does.
   *
   * This can be used by clients or servers to provide context about their purpose
   * and capabilities. For example, a server might describe the types of resources
   * or tools it provides, while a client might describe its intended use case.
   */
  description: u().optional()
}), aa = et(h({
  applyDefaults: A().optional()
}), I(u(), C())), ca = $n((e) => e && typeof e == "object" && !Array.isArray(e) && Object.keys(e).length === 0 ? { form: {} } : e, et(h({
  form: aa.optional(),
  url: O.optional()
}), I(u(), C()).optional())), ua = D({
  /**
   * Present if the client supports listing tasks.
   */
  list: O.optional(),
  /**
   * Present if the client supports cancelling tasks.
   */
  cancel: O.optional(),
  /**
   * Capabilities for task creation on specific request types.
   */
  requests: D({
    /**
     * Task support for sampling requests.
     */
    sampling: D({
      createMessage: O.optional()
    }).optional(),
    /**
     * Task support for elicitation requests.
     */
    elicitation: D({
      create: O.optional()
    }).optional()
  }).optional()
}), la = D({
  /**
   * Present if the server supports listing tasks.
   */
  list: O.optional(),
  /**
   * Present if the server supports cancelling tasks.
   */
  cancel: O.optional(),
  /**
   * Capabilities for task creation on specific request types.
   */
  requests: D({
    /**
     * Task support for tool requests.
     */
    tools: D({
      call: O.optional()
    }).optional()
  }).optional()
}), pa = h({
  /**
   * Experimental, non-standard capabilities that the client supports.
   */
  experimental: I(u(), O).optional(),
  /**
   * Present if the client supports sampling from an LLM.
   */
  sampling: h({
    /**
     * Present if the client supports context inclusion via includeContext parameter.
     * If not declared, servers SHOULD only use `includeContext: "none"` (or omit it).
     */
    context: O.optional(),
    /**
     * Present if the client supports tool use via tools and toolChoice parameters.
     */
    tools: O.optional()
  }).optional(),
  /**
   * Present if the client supports eliciting user input.
   */
  elicitation: ca.optional(),
  /**
   * Present if the client supports listing roots.
   */
  roots: h({
    /**
     * Whether the client supports issuing notifications for changes to the roots list.
     */
    listChanged: A().optional()
  }).optional(),
  /**
   * Present if the client supports task creation.
   */
  tasks: ua.optional(),
  /**
   * Extensions that the client supports. Keys are extension identifiers (vendor-prefix/extension-name).
   */
  extensions: I(u(), O).optional()
}), da = G.extend({
  /**
   * The latest version of the Model Context Protocol that the client supports. The client MAY decide to support older versions as well.
   */
  protocolVersion: u(),
  capabilities: pa,
  clientInfo: Zn
}), Pn = M.extend({
  method: v("initialize"),
  params: da
}), ma = h({
  /**
   * Experimental, non-standard capabilities that the server supports.
   */
  experimental: I(u(), O).optional(),
  /**
   * Present if the server supports sending log messages to the client.
   */
  logging: O.optional(),
  /**
   * Present if the server supports sending completions to the client.
   */
  completions: O.optional(),
  /**
   * Present if the server offers any prompt templates.
   */
  prompts: h({
    /**
     * Whether this server supports issuing notifications for changes to the prompt list.
     */
    listChanged: A().optional()
  }).optional(),
  /**
   * Present if the server offers any resources to read.
   */
  resources: h({
    /**
     * Whether this server supports clients subscribing to resource updates.
     */
    subscribe: A().optional(),
    /**
     * Whether this server supports issuing notifications for changes to the resource list.
     */
    listChanged: A().optional()
  }).optional(),
  /**
   * Present if the server offers any tools to call.
   */
  tools: h({
    /**
     * Whether this server supports issuing notifications for changes to the tool list.
     */
    listChanged: A().optional()
  }).optional(),
  /**
   * Present if the server supports task creation.
   */
  tasks: la.optional(),
  /**
   * Extensions that the server supports. Keys are extension identifiers (vendor-prefix/extension-name).
   */
  extensions: I(u(), O).optional()
}), ha = L.extend({
  /**
   * The version of the Model Context Protocol that the server wants to use. This may not match the version that the client requested. If the client cannot support this version, it MUST disconnect.
   */
  protocolVersion: u(),
  capabilities: ma,
  serverInfo: Zn,
  /**
   * Instructions describing how to use the server and its features.
   *
   * This can be used by clients to improve the LLM's understanding of available tools, resources, etc. It can be thought of like a "hint" to the model. For example, this information MAY be added to the system prompt.
   */
  instructions: u().optional()
}), qn = J.extend({
  method: v("notifications/initialized"),
  params: V.optional()
}), it = M.extend({
  method: v("ping"),
  params: G.optional()
}), fa = h({
  /**
   * The progress thus far. This should increase every time progress is made, even if the total is unknown.
   */
  progress: $(),
  /**
   * Total number of items to process (or total progress required), if known.
   */
  total: q($()),
  /**
   * An optional message describing the current progress.
   */
  message: q(u())
}), ga = h({
  ...V.shape,
  ...fa.shape,
  /**
   * The progress token which was given in the initial request, used to associate this notification with the request that is proceeding.
   */
  progressToken: zn
}), at = J.extend({
  method: v("notifications/progress"),
  params: ga
}), _a = G.extend({
  /**
   * An opaque token representing the current pagination position.
   * If provided, the server should return results starting after this cursor.
   */
  cursor: In.optional()
}), ve = M.extend({
  params: _a.optional()
}), be = L.extend({
  /**
   * An opaque token representing the pagination position after the last returned result.
   * If present, there may be more results available.
   */
  nextCursor: In.optional()
}), va = H(["working", "input_required", "completed", "failed", "cancelled"]), we = h({
  taskId: u(),
  status: va,
  /**
   * Time in milliseconds to keep task results available after completion.
   * If null, the task has unlimited lifetime until manually cleaned up.
   */
  ttl: P([$(), zi()]),
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: u(),
  /**
   * ISO 8601 timestamp when the task was last updated.
   */
  lastUpdatedAt: u(),
  pollInterval: q($()),
  /**
   * Optional diagnostic message for failed tasks or other status information.
   */
  statusMessage: q(u())
}), De = L.extend({
  task: we
}), ba = V.merge(we), Pe = J.extend({
  method: v("notifications/tasks/status"),
  params: ba
}), ct = M.extend({
  method: v("tasks/get"),
  params: G.extend({
    taskId: u()
  })
}), ut = L.merge(we), lt = M.extend({
  method: v("tasks/result"),
  params: G.extend({
    taskId: u()
  })
});
L.loose();
const pt = ve.extend({
  method: v("tasks/list")
}), dt = be.extend({
  tasks: T(we)
}), mt = M.extend({
  method: v("tasks/cancel"),
  params: G.extend({
    taskId: u()
  })
}), wa = L.merge(we), Cn = h({
  /**
   * The URI of this resource.
   */
  uri: u(),
  /**
   * The MIME type of this resource, if known.
   */
  mimeType: q(u()),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), jn = Cn.extend({
  /**
   * The text of the item. This must only be set if the item can actually be represented as text (not binary data).
   */
  text: u()
}), ht = u().refine((e) => {
  try {
    return atob(e), !0;
  } catch {
    return !1;
  }
}, { message: "Invalid Base64 string" }), Nn = Cn.extend({
  /**
   * A base64-encoded string representing the binary data of the item.
   */
  blob: ht
}), ke = H(["user", "assistant"]), se = h({
  /**
   * Intended audience(s) for the resource.
   */
  audience: T(ke).optional(),
  /**
   * Importance hint for the resource, from 0 (least) to 1 (most).
   */
  priority: $().min(0).max(1).optional(),
  /**
   * ISO 8601 timestamp for the most recent modification.
   */
  lastModified: fn({ offset: !0 }).optional()
}), On = h({
  ...re.shape,
  ..._e.shape,
  /**
   * The URI of this resource.
   */
  uri: u(),
  /**
   * A description of what this resource represents.
   *
   * This can be used by clients to improve the LLM's understanding of available resources. It can be thought of like a "hint" to the model.
   */
  description: q(u()),
  /**
   * The MIME type of this resource, if known.
   */
  mimeType: q(u()),
  /**
   * The size of the raw resource content, in bytes (i.e., before base64 encoding or any tokenization), if known.
   *
   * This can be used by Hosts to display file sizes and estimate context window usage.
   */
  size: q($()),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: q(D({}))
}), ka = h({
  ...re.shape,
  ..._e.shape,
  /**
   * A URI template (according to RFC 6570) that can be used to construct resource URIs.
   */
  uriTemplate: u(),
  /**
   * A description of what this template is for.
   *
   * This can be used by clients to improve the LLM's understanding of available resources. It can be thought of like a "hint" to the model.
   */
  description: q(u()),
  /**
   * The MIME type for all resources that match this template. This should only be included if all resources matching this template have the same type.
   */
  mimeType: q(u()),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: q(D({}))
}), An = ve.extend({
  method: v("resources/list")
}), ya = be.extend({
  resources: T(On)
}), Sa = ve.extend({
  method: v("resources/templates/list")
}), Ta = be.extend({
  resourceTemplates: T(ka)
}), ft = G.extend({
  /**
   * The URI of the resource to read. The URI can use any protocol; it is up to the server how to interpret it.
   *
   * @format uri
   */
  uri: u()
}), $a = ft, Mn = M.extend({
  method: v("resources/read"),
  params: $a
}), Ra = L.extend({
  contents: T(P([jn, Nn]))
}), za = J.extend({
  method: v("notifications/resources/list_changed"),
  params: V.optional()
}), Ia = ft, Ea = M.extend({
  method: v("resources/subscribe"),
  params: Ia
}), xa = ft, Za = M.extend({
  method: v("resources/unsubscribe"),
  params: xa
}), Pa = V.extend({
  /**
   * The URI of the resource that has been updated. This might be a sub-resource of the one that the client actually subscribed to.
   */
  uri: u()
}), qa = J.extend({
  method: v("notifications/resources/updated"),
  params: Pa
}), Ca = h({
  /**
   * The name of the argument.
   */
  name: u(),
  /**
   * A human-readable description of the argument.
   */
  description: q(u()),
  /**
   * Whether this argument must be provided.
   */
  required: q(A())
}), ja = h({
  ...re.shape,
  ..._e.shape,
  /**
   * An optional description of what this prompt provides
   */
  description: q(u()),
  /**
   * A list of arguments to use for templating the prompt.
   */
  arguments: q(T(Ca)),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: q(D({}))
}), Ln = ve.extend({
  method: v("prompts/list")
}), Na = be.extend({
  prompts: T(ja)
}), Oa = G.extend({
  /**
   * The name of the prompt or prompt template.
   */
  name: u(),
  /**
   * Arguments to use for templating the prompt.
   */
  arguments: I(u(), u()).optional()
}), Un = M.extend({
  method: v("prompts/get"),
  params: Oa
}), gt = h({
  type: v("text"),
  /**
   * The text content of the message.
   */
  text: u(),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), _t = h({
  type: v("image"),
  /**
   * The base64-encoded image data.
   */
  data: ht,
  /**
   * The MIME type of the image. Different providers may support different image types.
   */
  mimeType: u(),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), vt = h({
  type: v("audio"),
  /**
   * The base64-encoded audio data.
   */
  data: ht,
  /**
   * The MIME type of the audio. Different providers may support different audio types.
   */
  mimeType: u(),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), Aa = h({
  type: v("tool_use"),
  /**
   * The name of the tool to invoke.
   * Must match a tool name from the request's tools array.
   */
  name: u(),
  /**
   * Unique identifier for this tool call.
   * Used to correlate with ToolResultContent in subsequent messages.
   */
  id: u(),
  /**
   * Arguments to pass to the tool.
   * Must conform to the tool's inputSchema.
   */
  input: I(u(), C()),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), Ma = h({
  type: v("resource"),
  resource: P([jn, Nn]),
  /**
   * Optional annotations for the client.
   */
  annotations: se.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), La = On.extend({
  type: v("resource_link")
}), bt = P([
  gt,
  _t,
  vt,
  La,
  Ma
]), Ua = h({
  role: ke,
  content: bt
}), Da = L.extend({
  /**
   * An optional description for the prompt.
   */
  description: u().optional(),
  messages: T(Ua)
}), Fa = J.extend({
  method: v("notifications/prompts/list_changed"),
  params: V.optional()
}), Ha = h({
  /**
   * A human-readable title for the tool.
   */
  title: u().optional(),
  /**
   * If true, the tool does not modify its environment.
   *
   * Default: false
   */
  readOnlyHint: A().optional(),
  /**
   * If true, the tool may perform destructive updates to its environment.
   * If false, the tool performs only additive updates.
   *
   * (This property is meaningful only when `readOnlyHint == false`)
   *
   * Default: true
   */
  destructiveHint: A().optional(),
  /**
   * If true, calling the tool repeatedly with the same arguments
   * will have no additional effect on the its environment.
   *
   * (This property is meaningful only when `readOnlyHint == false`)
   *
   * Default: false
   */
  idempotentHint: A().optional(),
  /**
   * If true, this tool may interact with an "open world" of external
   * entities. If false, the tool's domain of interaction is closed.
   * For example, the world of a web search tool is open, whereas that
   * of a memory tool is not.
   *
   * Default: true
   */
  openWorldHint: A().optional()
}), Ga = h({
  /**
   * Indicates the tool's preference for task-augmented execution.
   * - "required": Clients MUST invoke the tool as a task
   * - "optional": Clients MAY invoke the tool as a task or normal request
   * - "forbidden": Clients MUST NOT attempt to invoke the tool as a task
   *
   * If not present, defaults to "forbidden".
   */
  taskSupport: H(["required", "optional", "forbidden"]).optional()
}), Dn = h({
  ...re.shape,
  ..._e.shape,
  /**
   * A human-readable description of the tool.
   */
  description: u().optional(),
  /**
   * A JSON Schema 2020-12 object defining the expected parameters for the tool.
   * Must have type: 'object' at the root level per MCP spec.
   */
  inputSchema: h({
    type: v("object"),
    properties: I(u(), O).optional(),
    required: T(u()).optional()
  }).catchall(C()),
  /**
   * An optional JSON Schema 2020-12 object defining the structure of the tool's output
   * returned in the structuredContent field of a CallToolResult.
   * Must have type: 'object' at the root level per MCP spec.
   */
  outputSchema: h({
    type: v("object"),
    properties: I(u(), O).optional(),
    required: T(u()).optional()
  }).catchall(C()).optional(),
  /**
   * Optional additional tool information.
   */
  annotations: Ha.optional(),
  /**
   * Execution-related properties for this tool.
   */
  execution: Ga.optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), Fn = ve.extend({
  method: v("tools/list")
}), Va = be.extend({
  tools: T(Dn)
}), wt = L.extend({
  /**
   * A list of content objects that represent the result of the tool call.
   *
   * If the Tool does not define an outputSchema, this field MUST be present in the result.
   * For backwards compatibility, this field is always present, but it may be empty.
   */
  content: T(bt).default([]),
  /**
   * An object containing structured tool output.
   *
   * If the Tool defines an outputSchema, this field MUST be present in the result, and contain a JSON object that matches the schema.
   */
  structuredContent: I(u(), C()).optional(),
  /**
   * Whether the tool call ended in an error.
   *
   * If not set, this is assumed to be false (the call was successful).
   *
   * Any errors that originate from the tool SHOULD be reported inside the result
   * object, with `isError` set to true, _not_ as an MCP protocol-level error
   * response. Otherwise, the LLM would not be able to see that an error occurred
   * and self-correct.
   *
   * However, any errors in _finding_ the tool, an error indicating that the
   * server does not support tool calls, or any other exceptional conditions,
   * should be reported as an MCP error response.
   */
  isError: A().optional()
});
wt.or(L.extend({
  toolResult: C()
}));
const Ja = ge.extend({
  /**
   * The name of the tool to call.
   */
  name: u(),
  /**
   * Arguments to pass to the tool.
   */
  arguments: I(u(), C()).optional()
}), kt = M.extend({
  method: v("tools/call"),
  params: Ja
}), Ba = J.extend({
  method: v("notifications/tools/list_changed"),
  params: V.optional()
});
h({
  /**
   * If true, the list will be refreshed automatically when a list changed notification is received.
   * The callback will be called with the updated list.
   *
   * If false, the callback will be called with null items, allowing manual refresh.
   *
   * @default true
   */
  autoRefresh: A().default(!0),
  /**
   * Debounce time in milliseconds for list changed notification processing.
   *
   * Multiple notifications received within this timeframe will only trigger one refresh.
   * Set to 0 to disable debouncing.
   *
   * @default 300
   */
  debounceMs: $().int().nonnegative().default(300)
});
const qe = H(["debug", "info", "notice", "warning", "error", "critical", "alert", "emergency"]), Wa = G.extend({
  /**
   * The level of logging that the client wants to receive from the server. The server should send all logs at this level and higher (i.e., more severe) to the client as notifications/logging/message.
   */
  level: qe
}), Hn = M.extend({
  method: v("logging/setLevel"),
  params: Wa
}), Qa = V.extend({
  /**
   * The severity of this log message.
   */
  level: qe,
  /**
   * An optional name of the logger issuing this message.
   */
  logger: u().optional(),
  /**
   * The data to be logged, such as a string message or an object. Any JSON serializable type is allowed here.
   */
  data: C()
}), Ka = J.extend({
  method: v("notifications/message"),
  params: Qa
}), Ya = h({
  /**
   * A hint for a model name.
   */
  name: u().optional()
}), Xa = h({
  /**
   * Optional hints to use for model selection.
   */
  hints: T(Ya).optional(),
  /**
   * How much to prioritize cost when selecting a model.
   */
  costPriority: $().min(0).max(1).optional(),
  /**
   * How much to prioritize sampling speed (latency) when selecting a model.
   */
  speedPriority: $().min(0).max(1).optional(),
  /**
   * How much to prioritize intelligence and capabilities when selecting a model.
   */
  intelligencePriority: $().min(0).max(1).optional()
}), ec = h({
  /**
   * Controls when tools are used:
   * - "auto": Model decides whether to use tools (default)
   * - "required": Model MUST use at least one tool before completing
   * - "none": Model MUST NOT use any tools
   */
  mode: H(["auto", "required", "none"]).optional()
}), tc = h({
  type: v("tool_result"),
  toolUseId: u().describe("The unique identifier for the corresponding tool call."),
  content: T(bt).default([]),
  structuredContent: h({}).loose().optional(),
  isError: A().optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), nc = wn("type", [gt, _t, vt]), Ce = wn("type", [
  gt,
  _t,
  vt,
  Aa,
  tc
]), oc = h({
  role: ke,
  content: P([Ce, T(Ce)]),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), rc = ge.extend({
  messages: T(oc),
  /**
   * The server's preferences for which model to select. The client MAY modify or omit this request.
   */
  modelPreferences: Xa.optional(),
  /**
   * An optional system prompt the server wants to use for sampling. The client MAY modify or omit this prompt.
   */
  systemPrompt: u().optional(),
  /**
   * A request to include context from one or more MCP servers (including the caller), to be attached to the prompt.
   * The client MAY ignore this request.
   *
   * Default is "none". Values "thisServer" and "allServers" are soft-deprecated. Servers SHOULD only use these values if the client
   * declares ClientCapabilities.sampling.context. These values may be removed in future spec releases.
   */
  includeContext: H(["none", "thisServer", "allServers"]).optional(),
  temperature: $().optional(),
  /**
   * The requested maximum number of tokens to sample (to prevent runaway completions).
   *
   * The client MAY choose to sample fewer tokens than the requested maximum.
   */
  maxTokens: $().int(),
  stopSequences: T(u()).optional(),
  /**
   * Optional metadata to pass through to the LLM provider. The format of this metadata is provider-specific.
   */
  metadata: O.optional(),
  /**
   * Tools that the model may use during generation.
   * The client MUST return an error if this field is provided but ClientCapabilities.sampling.tools is not declared.
   */
  tools: T(Dn).optional(),
  /**
   * Controls how the model uses tools.
   * The client MUST return an error if this field is provided but ClientCapabilities.sampling.tools is not declared.
   * Default is `{ mode: "auto" }`.
   */
  toolChoice: ec.optional()
}), sc = M.extend({
  method: v("sampling/createMessage"),
  params: rc
}), yt = L.extend({
  /**
   * The name of the model that generated the message.
   */
  model: u(),
  /**
   * The reason why sampling stopped, if known.
   *
   * Standard values:
   * - "endTurn": Natural end of the assistant's turn
   * - "stopSequence": A stop sequence was encountered
   * - "maxTokens": Maximum token limit was reached
   *
   * This field is an open string to allow for provider-specific stop reasons.
   */
  stopReason: q(H(["endTurn", "stopSequence", "maxTokens"]).or(u())),
  role: ke,
  /**
   * Response content. Single content block (text, image, or audio).
   */
  content: nc
}), Gn = L.extend({
  /**
   * The name of the model that generated the message.
   */
  model: u(),
  /**
   * The reason why sampling stopped, if known.
   *
   * Standard values:
   * - "endTurn": Natural end of the assistant's turn
   * - "stopSequence": A stop sequence was encountered
   * - "maxTokens": Maximum token limit was reached
   * - "toolUse": The model wants to use one or more tools
   *
   * This field is an open string to allow for provider-specific stop reasons.
   */
  stopReason: q(H(["endTurn", "stopSequence", "maxTokens", "toolUse"]).or(u())),
  role: ke,
  /**
   * Response content. May be a single block or array. May include ToolUseContent if stopReason is "toolUse".
   */
  content: P([Ce, T(Ce)])
}), ic = h({
  type: v("boolean"),
  title: u().optional(),
  description: u().optional(),
  default: A().optional()
}), ac = h({
  type: v("string"),
  title: u().optional(),
  description: u().optional(),
  minLength: $().optional(),
  maxLength: $().optional(),
  format: H(["email", "uri", "date", "date-time"]).optional(),
  default: u().optional()
}), cc = h({
  type: H(["number", "integer"]),
  title: u().optional(),
  description: u().optional(),
  minimum: $().optional(),
  maximum: $().optional(),
  default: $().optional()
}), uc = h({
  type: v("string"),
  title: u().optional(),
  description: u().optional(),
  enum: T(u()),
  default: u().optional()
}), lc = h({
  type: v("string"),
  title: u().optional(),
  description: u().optional(),
  oneOf: T(h({
    const: u(),
    title: u()
  })),
  default: u().optional()
}), pc = h({
  type: v("string"),
  title: u().optional(),
  description: u().optional(),
  enum: T(u()),
  enumNames: T(u()).optional(),
  default: u().optional()
}), dc = P([uc, lc]), mc = h({
  type: v("array"),
  title: u().optional(),
  description: u().optional(),
  minItems: $().optional(),
  maxItems: $().optional(),
  items: h({
    type: v("string"),
    enum: T(u())
  }),
  default: T(u()).optional()
}), hc = h({
  type: v("array"),
  title: u().optional(),
  description: u().optional(),
  minItems: $().optional(),
  maxItems: $().optional(),
  items: h({
    anyOf: T(h({
      const: u(),
      title: u()
    }))
  }),
  default: T(u()).optional()
}), fc = P([mc, hc]), gc = P([pc, dc, fc]), _c = P([gc, ic, ac, cc]), vc = ge.extend({
  /**
   * The elicitation mode.
   *
   * Optional for backward compatibility. Clients MUST treat missing mode as "form".
   */
  mode: v("form").optional(),
  /**
   * The message to present to the user describing what information is being requested.
   */
  message: u(),
  /**
   * A restricted subset of JSON Schema.
   * Only top-level properties are allowed, without nesting.
   */
  requestedSchema: h({
    type: v("object"),
    properties: I(u(), _c),
    required: T(u()).optional()
  })
}), bc = ge.extend({
  /**
   * The elicitation mode.
   */
  mode: v("url"),
  /**
   * The message to present to the user explaining why the interaction is needed.
   */
  message: u(),
  /**
   * The ID of the elicitation, which must be unique within the context of the server.
   * The client MUST treat this ID as an opaque value.
   */
  elicitationId: u(),
  /**
   * The URL that the user should navigate to.
   */
  url: u().url()
}), wc = P([vc, bc]), kc = M.extend({
  method: v("elicitation/create"),
  params: wc
}), yc = V.extend({
  /**
   * The ID of the elicitation that completed.
   */
  elicitationId: u()
}), Sc = J.extend({
  method: v("notifications/elicitation/complete"),
  params: yc
}), je = L.extend({
  /**
   * The user action in response to the elicitation.
   * - "accept": User submitted the form/confirmed the action
   * - "decline": User explicitly decline the action
   * - "cancel": User dismissed without making an explicit choice
   */
  action: H(["accept", "decline", "cancel"]),
  /**
   * The submitted form data, only present when action is "accept".
   * Contains values matching the requested schema.
   * Per MCP spec, content is "typically omitted" for decline/cancel actions.
   * We normalize null to undefined for leniency while maintaining type compatibility.
   */
  content: $n((e) => e === null ? void 0 : e, I(u(), P([u(), $(), A(), T(u())])).optional())
}), Tc = h({
  type: v("ref/resource"),
  /**
   * The URI or URI template of the resource.
   */
  uri: u()
}), $c = h({
  type: v("ref/prompt"),
  /**
   * The name of the prompt or prompt template
   */
  name: u()
}), Rc = G.extend({
  ref: P([$c, Tc]),
  /**
   * The argument's information
   */
  argument: h({
    /**
     * The name of the argument
     */
    name: u(),
    /**
     * The value of the argument to use for completion matching.
     */
    value: u()
  }),
  context: h({
    /**
     * Previously-resolved variables in a URI template or prompt.
     */
    arguments: I(u(), u()).optional()
  }).optional()
}), zc = M.extend({
  method: v("completion/complete"),
  params: Rc
}), Ic = L.extend({
  completion: D({
    /**
     * An array of completion values. Must not exceed 100 items.
     */
    values: T(u()).max(100),
    /**
     * The total number of completion options available. This can exceed the number of values actually sent in the response.
     */
    total: q($().int()),
    /**
     * Indicates whether there are additional completion options beyond those provided in the current response, even if the exact total is unknown.
     */
    hasMore: q(A())
  })
}), Ec = h({
  /**
   * The URI identifying the root. This *must* start with file:// for now.
   */
  uri: u().startsWith("file://"),
  /**
   * An optional name for the root.
   */
  name: u().optional(),
  /**
   * See [MCP specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/47339c03c143bb4ec01a26e721a1b8fe66634ebe/docs/specification/draft/basic/index.mdx#general-fields)
   * for notes on _meta usage.
   */
  _meta: I(u(), C()).optional()
}), xc = M.extend({
  method: v("roots/list"),
  params: G.optional()
}), Vn = L.extend({
  roots: T(Ec)
}), Zc = J.extend({
  method: v("notifications/roots/list_changed"),
  params: V.optional()
});
P([
  it,
  Pn,
  zc,
  Hn,
  Un,
  Ln,
  An,
  Sa,
  Mn,
  Ea,
  Za,
  kt,
  Fn,
  ct,
  lt,
  pt,
  mt
]);
P([
  st,
  at,
  qn,
  Zc,
  Pe
]);
P([
  rt,
  yt,
  Gn,
  je,
  Vn,
  ut,
  dt,
  De
]);
P([
  it,
  sc,
  kc,
  xc,
  ct,
  lt,
  pt,
  mt
]);
P([
  st,
  at,
  Ka,
  qa,
  za,
  Ba,
  Fa,
  Pe,
  Sc
]);
P([
  rt,
  ha,
  Ic,
  Da,
  Na,
  ya,
  Ta,
  Ra,
  wt,
  Va,
  ut,
  dt,
  De
]);
class w extends Error {
  constructor(t, n, o) {
    super(`MCP error ${t}: ${n}`), this.code = t, this.data = o, this.name = "McpError";
  }
  /**
   * Factory method to create the appropriate error type based on the error code and data
   */
  static fromError(t, n, o) {
    if (t === S.UrlElicitationRequired && o) {
      const r = o;
      if (r.elicitations)
        return new Pc(r.elicitations, n);
    }
    return new w(t, n, o);
  }
}
class Pc extends w {
  constructor(t, n = `URL elicitation${t.length > 1 ? "s" : ""} required`) {
    super(S.UrlElicitationRequired, n, {
      elicitations: t
    });
  }
  get elicitations() {
    return this.data?.elicitations ?? [];
  }
}
function Y(e) {
  return e === "completed" || e === "failed" || e === "cancelled";
}
new Set("ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvxyz0123456789");
function Ht(e) {
  const n = hn(e)?.method;
  if (!n)
    throw new Error("Schema is missing a method literal");
  const o = Js(n);
  if (typeof o != "string")
    throw new Error("Schema method literal must be a string");
  return o;
}
function Gt(e, t) {
  const n = pe(e, t);
  if (!n.success)
    throw n.error;
  return n.data;
}
const qc = 6e4;
class Cc {
  constructor(t) {
    this._options = t, this._requestMessageId = 0, this._requestHandlers = /* @__PURE__ */ new Map(), this._requestHandlerAbortControllers = /* @__PURE__ */ new Map(), this._notificationHandlers = /* @__PURE__ */ new Map(), this._responseHandlers = /* @__PURE__ */ new Map(), this._progressHandlers = /* @__PURE__ */ new Map(), this._timeoutInfo = /* @__PURE__ */ new Map(), this._pendingDebouncedNotifications = /* @__PURE__ */ new Set(), this._taskProgressTokens = /* @__PURE__ */ new Map(), this._requestResolvers = /* @__PURE__ */ new Map(), this.setNotificationHandler(st, (n) => {
      this._oncancel(n);
    }), this.setNotificationHandler(at, (n) => {
      this._onprogress(n);
    }), this.setRequestHandler(
      it,
      // Automatic pong by default.
      (n) => ({})
    ), this._taskStore = t?.taskStore, this._taskMessageQueue = t?.taskMessageQueue, this._taskStore && (this.setRequestHandler(ct, async (n, o) => {
      const r = await this._taskStore.getTask(n.params.taskId, o.sessionId);
      if (!r)
        throw new w(S.InvalidParams, "Failed to retrieve task: Task not found");
      return {
        ...r
      };
    }), this.setRequestHandler(lt, async (n, o) => {
      const r = async () => {
        const s = n.params.taskId;
        if (this._taskMessageQueue) {
          let c;
          for (; c = await this._taskMessageQueue.dequeue(s, o.sessionId); ) {
            if (c.type === "response" || c.type === "error") {
              const p = c.message, d = p.id, f = this._requestResolvers.get(d);
              if (f)
                if (this._requestResolvers.delete(d), c.type === "response")
                  f(p);
                else {
                  const m = p, g = new w(m.error.code, m.error.message, m.error.data);
                  f(g);
                }
              else {
                const m = c.type === "response" ? "Response" : "Error";
                this._onerror(new Error(`${m} handler missing for request ${d}`));
              }
              continue;
            }
            await this._transport?.send(c.message, { relatedRequestId: o.requestId });
          }
        }
        const i = await this._taskStore.getTask(s, o.sessionId);
        if (!i)
          throw new w(S.InvalidParams, `Task not found: ${s}`);
        if (!Y(i.status))
          return await this._waitForTaskUpdate(s, o.signal), await r();
        if (Y(i.status)) {
          const c = await this._taskStore.getTaskResult(s, o.sessionId);
          return this._clearTaskQueue(s), {
            ...c,
            _meta: {
              ...c._meta,
              [X]: {
                taskId: s
              }
            }
          };
        }
        return await r();
      };
      return await r();
    }), this.setRequestHandler(pt, async (n, o) => {
      try {
        const { tasks: r, nextCursor: s } = await this._taskStore.listTasks(n.params?.cursor, o.sessionId);
        return {
          tasks: r,
          nextCursor: s,
          _meta: {}
        };
      } catch (r) {
        throw new w(S.InvalidParams, `Failed to list tasks: ${r instanceof Error ? r.message : String(r)}`);
      }
    }), this.setRequestHandler(mt, async (n, o) => {
      try {
        const r = await this._taskStore.getTask(n.params.taskId, o.sessionId);
        if (!r)
          throw new w(S.InvalidParams, `Task not found: ${n.params.taskId}`);
        if (Y(r.status))
          throw new w(S.InvalidParams, `Cannot cancel task in terminal status: ${r.status}`);
        await this._taskStore.updateTaskStatus(n.params.taskId, "cancelled", "Client cancelled task execution.", o.sessionId), this._clearTaskQueue(n.params.taskId);
        const s = await this._taskStore.getTask(n.params.taskId, o.sessionId);
        if (!s)
          throw new w(S.InvalidParams, `Task not found after cancellation: ${n.params.taskId}`);
        return {
          _meta: {},
          ...s
        };
      } catch (r) {
        throw r instanceof w ? r : new w(S.InvalidRequest, `Failed to cancel task: ${r instanceof Error ? r.message : String(r)}`);
      }
    }));
  }
  async _oncancel(t) {
    if (!t.params.requestId)
      return;
    this._requestHandlerAbortControllers.get(t.params.requestId)?.abort(t.params.reason);
  }
  _setupTimeout(t, n, o, r, s = !1) {
    this._timeoutInfo.set(t, {
      timeoutId: setTimeout(r, n),
      startTime: Date.now(),
      timeout: n,
      maxTotalTimeout: o,
      resetTimeoutOnProgress: s,
      onTimeout: r
    });
  }
  _resetTimeout(t) {
    const n = this._timeoutInfo.get(t);
    if (!n)
      return !1;
    const o = Date.now() - n.startTime;
    if (n.maxTotalTimeout && o >= n.maxTotalTimeout)
      throw this._timeoutInfo.delete(t), w.fromError(S.RequestTimeout, "Maximum total timeout exceeded", {
        maxTotalTimeout: n.maxTotalTimeout,
        totalElapsed: o
      });
    return clearTimeout(n.timeoutId), n.timeoutId = setTimeout(n.onTimeout, n.timeout), !0;
  }
  _cleanupTimeout(t) {
    const n = this._timeoutInfo.get(t);
    n && (clearTimeout(n.timeoutId), this._timeoutInfo.delete(t));
  }
  /**
   * Attaches to the given transport, starts it, and starts listening for messages.
   *
   * The Protocol object assumes ownership of the Transport, replacing any callbacks that have already been set, and expects that it is the only user of the Transport instance going forward.
   */
  async connect(t) {
    if (this._transport)
      throw new Error("Already connected to a transport. Call close() before connecting to a new transport, or use a separate Protocol instance per connection.");
    this._transport = t;
    const n = this.transport?.onclose;
    this._transport.onclose = () => {
      n?.(), this._onclose();
    };
    const o = this.transport?.onerror;
    this._transport.onerror = (s) => {
      o?.(s), this._onerror(s);
    };
    const r = this._transport?.onmessage;
    this._transport.onmessage = (s, i) => {
      r?.(s, i), ze(s) || oa(s) ? this._onresponse(s) : Ft(s) ? this._onrequest(s, i) : na(s) ? this._onnotification(s) : this._onerror(new Error(`Unknown message type: ${JSON.stringify(s)}`));
    }, await this._transport.start();
  }
  _onclose() {
    const t = this._responseHandlers;
    this._responseHandlers = /* @__PURE__ */ new Map(), this._progressHandlers.clear(), this._taskProgressTokens.clear(), this._pendingDebouncedNotifications.clear();
    for (const o of this._timeoutInfo.values())
      clearTimeout(o.timeoutId);
    this._timeoutInfo.clear();
    for (const o of this._requestHandlerAbortControllers.values())
      o.abort();
    this._requestHandlerAbortControllers.clear();
    const n = w.fromError(S.ConnectionClosed, "Connection closed");
    this._transport = void 0, this.onclose?.();
    for (const o of t.values())
      o(n);
  }
  _onerror(t) {
    this.onerror?.(t);
  }
  _onnotification(t) {
    const n = this._notificationHandlers.get(t.method) ?? this.fallbackNotificationHandler;
    n !== void 0 && Promise.resolve().then(() => n(t)).catch((o) => this._onerror(new Error(`Uncaught error in notification handler: ${o}`)));
  }
  _onrequest(t, n) {
    const o = this._requestHandlers.get(t.method) ?? this.fallbackRequestHandler, r = this._transport, s = t.params?._meta?.[X]?.taskId;
    if (o === void 0) {
      const f = {
        jsonrpc: "2.0",
        id: t.id,
        error: {
          code: S.MethodNotFound,
          message: "Method not found"
        }
      };
      s && this._taskMessageQueue ? this._enqueueTaskMessage(s, {
        type: "error",
        message: f,
        timestamp: Date.now()
      }, r?.sessionId).catch((m) => this._onerror(new Error(`Failed to enqueue error response: ${m}`))) : r?.send(f).catch((m) => this._onerror(new Error(`Failed to send an error response: ${m}`)));
      return;
    }
    const i = new AbortController();
    this._requestHandlerAbortControllers.set(t.id, i);
    const c = ta(t.params) ? t.params.task : void 0, p = this._taskStore ? this.requestTaskStore(t, r?.sessionId) : void 0, d = {
      signal: i.signal,
      sessionId: r?.sessionId,
      _meta: t.params?._meta,
      sendNotification: async (f) => {
        if (i.signal.aborted)
          return;
        const m = { relatedRequestId: t.id };
        s && (m.relatedTask = { taskId: s }), await this.notification(f, m);
      },
      sendRequest: async (f, m, g) => {
        if (i.signal.aborted)
          throw new w(S.ConnectionClosed, "Request was cancelled");
        const b = { ...g, relatedRequestId: t.id };
        s && !b.relatedTask && (b.relatedTask = { taskId: s });
        const R = b.relatedTask?.taskId ?? s;
        return R && p && await p.updateTaskStatus(R, "input_required"), await this.request(f, m, b);
      },
      authInfo: n?.authInfo,
      requestId: t.id,
      requestInfo: n?.requestInfo,
      taskId: s,
      taskStore: p,
      taskRequestedTtl: c?.ttl,
      closeSSEStream: n?.closeSSEStream,
      closeStandaloneSSEStream: n?.closeStandaloneSSEStream
    };
    Promise.resolve().then(() => {
      c && this.assertTaskHandlerCapability(t.method);
    }).then(() => o(t, d)).then(async (f) => {
      if (i.signal.aborted)
        return;
      const m = {
        result: f,
        jsonrpc: "2.0",
        id: t.id
      };
      s && this._taskMessageQueue ? await this._enqueueTaskMessage(s, {
        type: "response",
        message: m,
        timestamp: Date.now()
      }, r?.sessionId) : await r?.send(m);
    }, async (f) => {
      if (i.signal.aborted)
        return;
      const m = {
        jsonrpc: "2.0",
        id: t.id,
        error: {
          code: Number.isSafeInteger(f.code) ? f.code : S.InternalError,
          message: f.message ?? "Internal error",
          ...f.data !== void 0 && { data: f.data }
        }
      };
      s && this._taskMessageQueue ? await this._enqueueTaskMessage(s, {
        type: "error",
        message: m,
        timestamp: Date.now()
      }, r?.sessionId) : await r?.send(m);
    }).catch((f) => this._onerror(new Error(`Failed to send response: ${f}`))).finally(() => {
      this._requestHandlerAbortControllers.get(t.id) === i && this._requestHandlerAbortControllers.delete(t.id);
    });
  }
  _onprogress(t) {
    const { progressToken: n, ...o } = t.params, r = Number(n), s = this._progressHandlers.get(r);
    if (!s) {
      this._onerror(new Error(`Received a progress notification for an unknown token: ${JSON.stringify(t)}`));
      return;
    }
    const i = this._responseHandlers.get(r), c = this._timeoutInfo.get(r);
    if (c && i && c.resetTimeoutOnProgress)
      try {
        this._resetTimeout(r);
      } catch (p) {
        this._responseHandlers.delete(r), this._progressHandlers.delete(r), this._cleanupTimeout(r), i(p);
        return;
      }
    s(o);
  }
  _onresponse(t) {
    const n = Number(t.id), o = this._requestResolvers.get(n);
    if (o) {
      if (this._requestResolvers.delete(n), ze(t))
        o(t);
      else {
        const i = new w(t.error.code, t.error.message, t.error.data);
        o(i);
      }
      return;
    }
    const r = this._responseHandlers.get(n);
    if (r === void 0) {
      this._onerror(new Error(`Received a response for an unknown message ID: ${JSON.stringify(t)}`));
      return;
    }
    this._responseHandlers.delete(n), this._cleanupTimeout(n);
    let s = !1;
    if (ze(t) && t.result && typeof t.result == "object") {
      const i = t.result;
      if (i.task && typeof i.task == "object") {
        const c = i.task;
        typeof c.taskId == "string" && (s = !0, this._taskProgressTokens.set(c.taskId, n));
      }
    }
    if (s || this._progressHandlers.delete(n), ze(t))
      r(t);
    else {
      const i = w.fromError(t.error.code, t.error.message, t.error.data);
      r(i);
    }
  }
  get transport() {
    return this._transport;
  }
  /**
   * Closes the connection.
   */
  async close() {
    await this._transport?.close();
  }
  /**
   * Sends a request and returns an AsyncGenerator that yields response messages.
   * The generator is guaranteed to end with either a 'result' or 'error' message.
   *
   * @example
   * ```typescript
   * const stream = protocol.requestStream(request, resultSchema, options);
   * for await (const message of stream) {
   *   switch (message.type) {
   *     case 'taskCreated':
   *       console.log('Task created:', message.task.taskId);
   *       break;
   *     case 'taskStatus':
   *       console.log('Task status:', message.task.status);
   *       break;
   *     case 'result':
   *       console.log('Final result:', message.result);
   *       break;
   *     case 'error':
   *       console.error('Error:', message.error);
   *       break;
   *   }
   * }
   * ```
   *
   * @experimental Use `client.experimental.tasks.requestStream()` to access this method.
   */
  async *requestStream(t, n, o) {
    const { task: r } = o ?? {};
    if (!r) {
      try {
        yield { type: "result", result: await this.request(t, n, o) };
      } catch (i) {
        yield {
          type: "error",
          error: i instanceof w ? i : new w(S.InternalError, String(i))
        };
      }
      return;
    }
    let s;
    try {
      const i = await this.request(t, De, o);
      if (i.task)
        s = i.task.taskId, yield { type: "taskCreated", task: i.task };
      else
        throw new w(S.InternalError, "Task creation did not return a task");
      for (; ; ) {
        const c = await this.getTask({ taskId: s }, o);
        if (yield { type: "taskStatus", task: c }, Y(c.status)) {
          c.status === "completed" ? yield { type: "result", result: await this.getTaskResult({ taskId: s }, n, o) } : c.status === "failed" ? yield {
            type: "error",
            error: new w(S.InternalError, `Task ${s} failed`)
          } : c.status === "cancelled" && (yield {
            type: "error",
            error: new w(S.InternalError, `Task ${s} was cancelled`)
          });
          return;
        }
        if (c.status === "input_required") {
          yield { type: "result", result: await this.getTaskResult({ taskId: s }, n, o) };
          return;
        }
        const p = c.pollInterval ?? this._options?.defaultTaskPollInterval ?? 1e3;
        await new Promise((d) => setTimeout(d, p)), o?.signal?.throwIfAborted();
      }
    } catch (i) {
      yield {
        type: "error",
        error: i instanceof w ? i : new w(S.InternalError, String(i))
      };
    }
  }
  /**
   * Sends a request and waits for a response.
   *
   * Do not use this method to emit notifications! Use notification() instead.
   */
  request(t, n, o) {
    const { relatedRequestId: r, resumptionToken: s, onresumptiontoken: i, task: c, relatedTask: p } = o ?? {};
    return new Promise((d, f) => {
      const m = (y) => {
        f(y);
      };
      if (!this._transport) {
        m(new Error("Not connected"));
        return;
      }
      if (this._options?.enforceStrictCapabilities === !0)
        try {
          this.assertCapabilityForMethod(t.method), c && this.assertTaskCapability(t.method);
        } catch (y) {
          m(y);
          return;
        }
      o?.signal?.throwIfAborted();
      const g = this._requestMessageId++, b = {
        ...t,
        jsonrpc: "2.0",
        id: g
      };
      o?.onprogress && (this._progressHandlers.set(g, o.onprogress), b.params = {
        ...t.params,
        _meta: {
          ...t.params?._meta || {},
          progressToken: g
        }
      }), c && (b.params = {
        ...b.params,
        task: c
      }), p && (b.params = {
        ...b.params,
        _meta: {
          ...b.params?._meta || {},
          [X]: p
        }
      });
      const R = (y) => {
        this._responseHandlers.delete(g), this._progressHandlers.delete(g), this._cleanupTimeout(g), this._transport?.send({
          jsonrpc: "2.0",
          method: "notifications/cancelled",
          params: {
            requestId: g,
            reason: String(y)
          }
        }, { relatedRequestId: r, resumptionToken: s, onresumptiontoken: i }).catch((x) => this._onerror(new Error(`Failed to send cancellation: ${x}`)));
        const k = y instanceof w ? y : new w(S.RequestTimeout, String(y));
        f(k);
      };
      this._responseHandlers.set(g, (y) => {
        if (!o?.signal?.aborted) {
          if (y instanceof Error)
            return f(y);
          try {
            const k = pe(n, y.result);
            k.success ? d(k.data) : f(k.error);
          } catch (k) {
            f(k);
          }
        }
      }), o?.signal?.addEventListener("abort", () => {
        R(o?.signal?.reason);
      });
      const U = o?.timeout ?? qc, ae = () => R(w.fromError(S.RequestTimeout, "Request timed out", { timeout: U }));
      this._setupTimeout(g, U, o?.maxTotalTimeout, ae, o?.resetTimeoutOnProgress ?? !1);
      const K = p?.taskId;
      if (K) {
        const y = (k) => {
          const x = this._responseHandlers.get(g);
          x ? x(k) : this._onerror(new Error(`Response handler missing for side-channeled request ${g}`));
        };
        this._requestResolvers.set(g, y), this._enqueueTaskMessage(K, {
          type: "request",
          message: b,
          timestamp: Date.now()
        }).catch((k) => {
          this._cleanupTimeout(g), f(k);
        });
      } else
        this._transport.send(b, { relatedRequestId: r, resumptionToken: s, onresumptiontoken: i }).catch((y) => {
          this._cleanupTimeout(g), f(y);
        });
    });
  }
  /**
   * Gets the current status of a task.
   *
   * @experimental Use `client.experimental.tasks.getTask()` to access this method.
   */
  async getTask(t, n) {
    return this.request({ method: "tasks/get", params: t }, ut, n);
  }
  /**
   * Retrieves the result of a completed task.
   *
   * @experimental Use `client.experimental.tasks.getTaskResult()` to access this method.
   */
  async getTaskResult(t, n, o) {
    return this.request({ method: "tasks/result", params: t }, n, o);
  }
  /**
   * Lists tasks, optionally starting from a pagination cursor.
   *
   * @experimental Use `client.experimental.tasks.listTasks()` to access this method.
   */
  async listTasks(t, n) {
    return this.request({ method: "tasks/list", params: t }, dt, n);
  }
  /**
   * Cancels a specific task.
   *
   * @experimental Use `client.experimental.tasks.cancelTask()` to access this method.
   */
  async cancelTask(t, n) {
    return this.request({ method: "tasks/cancel", params: t }, wa, n);
  }
  /**
   * Emits a notification, which is a one-way message that does not expect a response.
   */
  async notification(t, n) {
    if (!this._transport)
      throw new Error("Not connected");
    this.assertNotificationCapability(t.method);
    const o = n?.relatedTask?.taskId;
    if (o) {
      const c = {
        ...t,
        jsonrpc: "2.0",
        params: {
          ...t.params,
          _meta: {
            ...t.params?._meta || {},
            [X]: n.relatedTask
          }
        }
      };
      await this._enqueueTaskMessage(o, {
        type: "notification",
        message: c,
        timestamp: Date.now()
      });
      return;
    }
    if ((this._options?.debouncedNotificationMethods ?? []).includes(t.method) && !t.params && !n?.relatedRequestId && !n?.relatedTask) {
      if (this._pendingDebouncedNotifications.has(t.method))
        return;
      this._pendingDebouncedNotifications.add(t.method), Promise.resolve().then(() => {
        if (this._pendingDebouncedNotifications.delete(t.method), !this._transport)
          return;
        let c = {
          ...t,
          jsonrpc: "2.0"
        };
        n?.relatedTask && (c = {
          ...c,
          params: {
            ...c.params,
            _meta: {
              ...c.params?._meta || {},
              [X]: n.relatedTask
            }
          }
        }), this._transport?.send(c, n).catch((p) => this._onerror(p));
      });
      return;
    }
    let i = {
      ...t,
      jsonrpc: "2.0"
    };
    n?.relatedTask && (i = {
      ...i,
      params: {
        ...i.params,
        _meta: {
          ...i.params?._meta || {},
          [X]: n.relatedTask
        }
      }
    }), await this._transport.send(i, n);
  }
  /**
   * Registers a handler to invoke when this protocol object receives a request with the given method.
   *
   * Note that this will replace any previous request handler for the same method.
   */
  setRequestHandler(t, n) {
    const o = Ht(t);
    this.assertRequestHandlerCapability(o), this._requestHandlers.set(o, (r, s) => {
      const i = Gt(t, r);
      return Promise.resolve(n(i, s));
    });
  }
  /**
   * Removes the request handler for the given method.
   */
  removeRequestHandler(t) {
    this._requestHandlers.delete(t);
  }
  /**
   * Asserts that a request handler has not already been set for the given method, in preparation for a new one being automatically installed.
   */
  assertCanSetRequestHandler(t) {
    if (this._requestHandlers.has(t))
      throw new Error(`A request handler for ${t} already exists, which would be overridden`);
  }
  /**
   * Registers a handler to invoke when this protocol object receives a notification with the given method.
   *
   * Note that this will replace any previous notification handler for the same method.
   */
  setNotificationHandler(t, n) {
    const o = Ht(t);
    this._notificationHandlers.set(o, (r) => {
      const s = Gt(t, r);
      return Promise.resolve(n(s));
    });
  }
  /**
   * Removes the notification handler for the given method.
   */
  removeNotificationHandler(t) {
    this._notificationHandlers.delete(t);
  }
  /**
   * Cleans up the progress handler associated with a task.
   * This should be called when a task reaches a terminal status.
   */
  _cleanupTaskProgressHandler(t) {
    const n = this._taskProgressTokens.get(t);
    n !== void 0 && (this._progressHandlers.delete(n), this._taskProgressTokens.delete(t));
  }
  /**
   * Enqueues a task-related message for side-channel delivery via tasks/result.
   * @param taskId The task ID to associate the message with
   * @param message The message to enqueue
   * @param sessionId Optional session ID for binding the operation to a specific session
   * @throws Error if taskStore is not configured or if enqueue fails (e.g., queue overflow)
   *
   * Note: If enqueue fails, it's the TaskMessageQueue implementation's responsibility to handle
   * the error appropriately (e.g., by failing the task, logging, etc.). The Protocol layer
   * simply propagates the error.
   */
  async _enqueueTaskMessage(t, n, o) {
    if (!this._taskStore || !this._taskMessageQueue)
      throw new Error("Cannot enqueue task message: taskStore and taskMessageQueue are not configured");
    const r = this._options?.maxTaskQueueSize;
    await this._taskMessageQueue.enqueue(t, n, o, r);
  }
  /**
   * Clears the message queue for a task and rejects any pending request resolvers.
   * @param taskId The task ID whose queue should be cleared
   * @param sessionId Optional session ID for binding the operation to a specific session
   */
  async _clearTaskQueue(t, n) {
    if (this._taskMessageQueue) {
      const o = await this._taskMessageQueue.dequeueAll(t, n);
      for (const r of o)
        if (r.type === "request" && Ft(r.message)) {
          const s = r.message.id, i = this._requestResolvers.get(s);
          i ? (i(new w(S.InternalError, "Task cancelled or completed")), this._requestResolvers.delete(s)) : this._onerror(new Error(`Resolver missing for request ${s} during task ${t} cleanup`));
        }
    }
  }
  /**
   * Waits for a task update (new messages or status change) with abort signal support.
   * Uses polling to check for updates at the task's configured poll interval.
   * @param taskId The task ID to wait for
   * @param signal Abort signal to cancel the wait
   * @returns Promise that resolves when an update occurs or rejects if aborted
   */
  async _waitForTaskUpdate(t, n) {
    let o = this._options?.defaultTaskPollInterval ?? 1e3;
    try {
      const r = await this._taskStore?.getTask(t);
      r?.pollInterval && (o = r.pollInterval);
    } catch {
    }
    return new Promise((r, s) => {
      if (n.aborted) {
        s(new w(S.InvalidRequest, "Request cancelled"));
        return;
      }
      const i = setTimeout(r, o);
      n.addEventListener("abort", () => {
        clearTimeout(i), s(new w(S.InvalidRequest, "Request cancelled"));
      }, { once: !0 });
    });
  }
  requestTaskStore(t, n) {
    const o = this._taskStore;
    if (!o)
      throw new Error("No task store configured");
    return {
      createTask: async (r) => {
        if (!t)
          throw new Error("No request provided");
        return await o.createTask(r, t.id, {
          method: t.method,
          params: t.params
        }, n);
      },
      getTask: async (r) => {
        const s = await o.getTask(r, n);
        if (!s)
          throw new w(S.InvalidParams, "Failed to retrieve task: Task not found");
        return s;
      },
      storeTaskResult: async (r, s, i) => {
        await o.storeTaskResult(r, s, i, n);
        const c = await o.getTask(r, n);
        if (c) {
          const p = Pe.parse({
            method: "notifications/tasks/status",
            params: c
          });
          await this.notification(p), Y(c.status) && this._cleanupTaskProgressHandler(r);
        }
      },
      getTaskResult: (r) => o.getTaskResult(r, n),
      updateTaskStatus: async (r, s, i) => {
        const c = await o.getTask(r, n);
        if (!c)
          throw new w(S.InvalidParams, `Task "${r}" not found - it may have been cleaned up`);
        if (Y(c.status))
          throw new w(S.InvalidParams, `Cannot update task "${r}" from terminal status "${c.status}" to "${s}". Terminal states (completed, failed, cancelled) cannot transition to other states.`);
        await o.updateTaskStatus(r, s, i, n);
        const p = await o.getTask(r, n);
        if (p) {
          const d = Pe.parse({
            method: "notifications/tasks/status",
            params: p
          });
          await this.notification(d), Y(p.status) && this._cleanupTaskProgressHandler(r);
        }
      },
      listTasks: (r) => o.listTasks(r, n)
    };
  }
}
function Vt(e) {
  return e !== null && typeof e == "object" && !Array.isArray(e);
}
function jc(e, t) {
  const n = { ...e };
  for (const o in t) {
    const r = o, s = t[r];
    if (s === void 0)
      continue;
    const i = n[r];
    Vt(i) && Vt(s) ? n[r] = { ...i, ...s } : n[r] = s;
  }
  return n;
}
function Nc() {
  const e = new Qn({
    strict: !1,
    validateFormats: !0,
    validateSchema: !1,
    allErrors: !0
  });
  return Kn(e), e;
}
class Oc {
  /**
   * Create an AJV validator
   *
   * @param ajv - Optional pre-configured AJV instance. If not provided, a default instance will be created.
   *
   * @example
   * ```typescript
   * // Use default configuration (recommended for most cases)
   * import { AjvJsonSchemaValidator } from '@modelcontextprotocol/sdk/validation/ajv';
   * const validator = new AjvJsonSchemaValidator();
   *
   * // Or provide custom AJV instance for advanced configuration
   * import { Ajv } from 'ajv';
   * import addFormats from 'ajv-formats';
   *
   * const ajv = new Ajv({ validateFormats: true });
   * addFormats(ajv);
   * const validator = new AjvJsonSchemaValidator(ajv);
   * ```
   */
  constructor(t) {
    this._ajv = t ?? Nc();
  }
  /**
   * Create a validator for the given JSON Schema
   *
   * The validator is compiled once and can be reused multiple times.
   * If the schema has an $id, it will be cached by AJV automatically.
   *
   * @param schema - Standard JSON Schema object
   * @returns A validator function that validates input data
   */
  getValidator(t) {
    const n = "$id" in t && typeof t.$id == "string" ? this._ajv.getSchema(t.$id) ?? this._ajv.compile(t) : this._ajv.compile(t);
    return (o) => n(o) ? {
      valid: !0,
      data: o,
      errorMessage: void 0
    } : {
      valid: !1,
      data: void 0,
      errorMessage: this._ajv.errorsText(n.errors)
    };
  }
}
class Ac {
  constructor(t) {
    this._server = t;
  }
  /**
   * Sends a request and returns an AsyncGenerator that yields response messages.
   * The generator is guaranteed to end with either a 'result' or 'error' message.
   *
   * This method provides streaming access to request processing, allowing you to
   * observe intermediate task status updates for task-augmented requests.
   *
   * @param request - The request to send
   * @param resultSchema - Zod schema for validating the result
   * @param options - Optional request options (timeout, signal, task creation params, etc.)
   * @returns AsyncGenerator that yields ResponseMessage objects
   *
   * @experimental
   */
  requestStream(t, n, o) {
    return this._server.requestStream(t, n, o);
  }
  /**
   * Sends a sampling request and returns an AsyncGenerator that yields response messages.
   * The generator is guaranteed to end with either a 'result' or 'error' message.
   *
   * For task-augmented requests, yields 'taskCreated' and 'taskStatus' messages
   * before the final result.
   *
   * @example
   * ```typescript
   * const stream = server.experimental.tasks.createMessageStream({
   *     messages: [{ role: 'user', content: { type: 'text', text: 'Hello' } }],
   *     maxTokens: 100
   * }, {
   *     onprogress: (progress) => {
   *         // Handle streaming tokens via progress notifications
   *         console.log('Progress:', progress.message);
   *     }
   * });
   *
   * for await (const message of stream) {
   *     switch (message.type) {
   *         case 'taskCreated':
   *             console.log('Task created:', message.task.taskId);
   *             break;
   *         case 'taskStatus':
   *             console.log('Task status:', message.task.status);
   *             break;
   *         case 'result':
   *             console.log('Final result:', message.result);
   *             break;
   *         case 'error':
   *             console.error('Error:', message.error);
   *             break;
   *     }
   * }
   * ```
   *
   * @param params - The sampling request parameters
   * @param options - Optional request options (timeout, signal, task creation params, onprogress, etc.)
   * @returns AsyncGenerator that yields ResponseMessage objects
   *
   * @experimental
   */
  createMessageStream(t, n) {
    const o = this._server.getClientCapabilities();
    if ((t.tools || t.toolChoice) && !o?.sampling?.tools)
      throw new Error("Client does not support sampling tools capability.");
    if (t.messages.length > 0) {
      const r = t.messages[t.messages.length - 1], s = Array.isArray(r.content) ? r.content : [r.content], i = s.some((f) => f.type === "tool_result"), c = t.messages.length > 1 ? t.messages[t.messages.length - 2] : void 0, p = c ? Array.isArray(c.content) ? c.content : [c.content] : [], d = p.some((f) => f.type === "tool_use");
      if (i) {
        if (s.some((f) => f.type !== "tool_result"))
          throw new Error("The last message must contain only tool_result content if any is present");
        if (!d)
          throw new Error("tool_result blocks are not matching any tool_use from the previous message");
      }
      if (d) {
        const f = new Set(p.filter((g) => g.type === "tool_use").map((g) => g.id)), m = new Set(s.filter((g) => g.type === "tool_result").map((g) => g.toolUseId));
        if (f.size !== m.size || ![...f].every((g) => m.has(g)))
          throw new Error("ids of tool_result blocks and tool_use blocks from previous message do not match");
      }
    }
    return this.requestStream({
      method: "sampling/createMessage",
      params: t
    }, yt, n);
  }
  /**
   * Sends an elicitation request and returns an AsyncGenerator that yields response messages.
   * The generator is guaranteed to end with either a 'result' or 'error' message.
   *
   * For task-augmented requests (especially URL-based elicitation), yields 'taskCreated'
   * and 'taskStatus' messages before the final result.
   *
   * @example
   * ```typescript
   * const stream = server.experimental.tasks.elicitInputStream({
   *     mode: 'url',
   *     message: 'Please authenticate',
   *     elicitationId: 'auth-123',
   *     url: 'https://example.com/auth'
   * }, {
   *     task: { ttl: 300000 } // Task-augmented for long-running auth flow
   * });
   *
   * for await (const message of stream) {
   *     switch (message.type) {
   *         case 'taskCreated':
   *             console.log('Task created:', message.task.taskId);
   *             break;
   *         case 'taskStatus':
   *             console.log('Task status:', message.task.status);
   *             break;
   *         case 'result':
   *             console.log('User action:', message.result.action);
   *             break;
   *         case 'error':
   *             console.error('Error:', message.error);
   *             break;
   *     }
   * }
   * ```
   *
   * @param params - The elicitation request parameters
   * @param options - Optional request options (timeout, signal, task creation params, etc.)
   * @returns AsyncGenerator that yields ResponseMessage objects
   *
   * @experimental
   */
  elicitInputStream(t, n) {
    const o = this._server.getClientCapabilities(), r = t.mode ?? "form";
    switch (r) {
      case "url": {
        if (!o?.elicitation?.url)
          throw new Error("Client does not support url elicitation.");
        break;
      }
      case "form": {
        if (!o?.elicitation?.form)
          throw new Error("Client does not support form elicitation.");
        break;
      }
    }
    const s = r === "form" && t.mode === void 0 ? { ...t, mode: "form" } : t;
    return this.requestStream({
      method: "elicitation/create",
      params: s
    }, je, n);
  }
  /**
   * Gets the current status of a task.
   *
   * @param taskId - The task identifier
   * @param options - Optional request options
   * @returns The task status
   *
   * @experimental
   */
  async getTask(t, n) {
    return this._server.getTask({ taskId: t }, n);
  }
  /**
   * Retrieves the result of a completed task.
   *
   * @param taskId - The task identifier
   * @param resultSchema - Zod schema for validating the result
   * @param options - Optional request options
   * @returns The task result
   *
   * @experimental
   */
  async getTaskResult(t, n, o) {
    return this._server.getTaskResult({ taskId: t }, n, o);
  }
  /**
   * Lists tasks with optional pagination.
   *
   * @param cursor - Optional pagination cursor
   * @param options - Optional request options
   * @returns List of tasks with optional next cursor
   *
   * @experimental
   */
  async listTasks(t, n) {
    return this._server.listTasks(t ? { cursor: t } : void 0, n);
  }
  /**
   * Cancels a running task.
   *
   * @param taskId - The task identifier
   * @param options - Optional request options
   *
   * @experimental
   */
  async cancelTask(t, n) {
    return this._server.cancelTask({ taskId: t }, n);
  }
}
function Mc(e, t, n) {
  if (!e)
    throw new Error(`${n} does not support task creation (required for ${t})`);
  switch (t) {
    case "tools/call":
      if (!e.tools?.call)
        throw new Error(`${n} does not support task creation for tools/call (required for ${t})`);
      break;
  }
}
function Lc(e, t, n) {
  if (!e)
    throw new Error(`${n} does not support task creation (required for ${t})`);
  switch (t) {
    case "sampling/createMessage":
      if (!e.sampling?.createMessage)
        throw new Error(`${n} does not support task creation for sampling/createMessage (required for ${t})`);
      break;
    case "elicitation/create":
      if (!e.elicitation?.create)
        throw new Error(`${n} does not support task creation for elicitation/create (required for ${t})`);
      break;
  }
}
class Uc extends Cc {
  /**
   * Initializes this server with the given name and version information.
   */
  constructor(t, n) {
    super(n), this._serverInfo = t, this._loggingLevels = /* @__PURE__ */ new Map(), this.LOG_LEVEL_SEVERITY = new Map(qe.options.map((o, r) => [o, r])), this.isMessageIgnored = (o, r) => {
      const s = this._loggingLevels.get(r);
      return s ? this.LOG_LEVEL_SEVERITY.get(o) < this.LOG_LEVEL_SEVERITY.get(s) : !1;
    }, this._capabilities = n?.capabilities ?? {}, this._instructions = n?.instructions, this._jsonSchemaValidator = n?.jsonSchemaValidator ?? new Oc(), this.setRequestHandler(Pn, (o) => this._oninitialize(o)), this.setNotificationHandler(qn, () => this.oninitialized?.()), this._capabilities.logging && this.setRequestHandler(Hn, async (o, r) => {
      const s = r.sessionId || r.requestInfo?.headers["mcp-session-id"] || void 0, { level: i } = o.params, c = qe.safeParse(i);
      return c.success && this._loggingLevels.set(s, c.data), {};
    });
  }
  /**
   * Access experimental features.
   *
   * WARNING: These APIs are experimental and may change without notice.
   *
   * @experimental
   */
  get experimental() {
    return this._experimental || (this._experimental = {
      tasks: new Ac(this)
    }), this._experimental;
  }
  /**
   * Registers new capabilities. This can only be called before connecting to a transport.
   *
   * The new capabilities will be merged with any existing capabilities previously given (e.g., at initialization).
   */
  registerCapabilities(t) {
    if (this.transport)
      throw new Error("Cannot register capabilities after connecting to transport");
    this._capabilities = jc(this._capabilities, t);
  }
  /**
   * Override request handler registration to enforce server-side validation for tools/call.
   */
  setRequestHandler(t, n) {
    const r = hn(t)?.method;
    if (!r)
      throw new Error("Schema is missing a method literal");
    let s;
    if (Ae(r)) {
      const c = r;
      s = c._zod?.def?.value ?? c.value;
    } else {
      const c = r;
      s = c._def?.value ?? c.value;
    }
    if (typeof s != "string")
      throw new Error("Schema method literal must be a string");
    if (s === "tools/call") {
      const c = async (p, d) => {
        const f = pe(kt, p);
        if (!f.success) {
          const R = f.error instanceof Error ? f.error.message : String(f.error);
          throw new w(S.InvalidParams, `Invalid tools/call request: ${R}`);
        }
        const { params: m } = f.data, g = await Promise.resolve(n(p, d));
        if (m.task) {
          const R = pe(De, g);
          if (!R.success) {
            const U = R.error instanceof Error ? R.error.message : String(R.error);
            throw new w(S.InvalidParams, `Invalid task creation result: ${U}`);
          }
          return R.data;
        }
        const b = pe(wt, g);
        if (!b.success) {
          const R = b.error instanceof Error ? b.error.message : String(b.error);
          throw new w(S.InvalidParams, `Invalid tools/call result: ${R}`);
        }
        return b.data;
      };
      return super.setRequestHandler(t, c);
    }
    return super.setRequestHandler(t, n);
  }
  assertCapabilityForMethod(t) {
    switch (t) {
      case "sampling/createMessage":
        if (!this._clientCapabilities?.sampling)
          throw new Error(`Client does not support sampling (required for ${t})`);
        break;
      case "elicitation/create":
        if (!this._clientCapabilities?.elicitation)
          throw new Error(`Client does not support elicitation (required for ${t})`);
        break;
      case "roots/list":
        if (!this._clientCapabilities?.roots)
          throw new Error(`Client does not support listing roots (required for ${t})`);
        break;
    }
  }
  assertNotificationCapability(t) {
    switch (t) {
      case "notifications/message":
        if (!this._capabilities.logging)
          throw new Error(`Server does not support logging (required for ${t})`);
        break;
      case "notifications/resources/updated":
      case "notifications/resources/list_changed":
        if (!this._capabilities.resources)
          throw new Error(`Server does not support notifying about resources (required for ${t})`);
        break;
      case "notifications/tools/list_changed":
        if (!this._capabilities.tools)
          throw new Error(`Server does not support notifying of tool list changes (required for ${t})`);
        break;
      case "notifications/prompts/list_changed":
        if (!this._capabilities.prompts)
          throw new Error(`Server does not support notifying of prompt list changes (required for ${t})`);
        break;
      case "notifications/elicitation/complete":
        if (!this._clientCapabilities?.elicitation?.url)
          throw new Error(`Client does not support URL elicitation (required for ${t})`);
        break;
    }
  }
  assertRequestHandlerCapability(t) {
    if (this._capabilities)
      switch (t) {
        case "completion/complete":
          if (!this._capabilities.completions)
            throw new Error(`Server does not support completions (required for ${t})`);
          break;
        case "logging/setLevel":
          if (!this._capabilities.logging)
            throw new Error(`Server does not support logging (required for ${t})`);
          break;
        case "prompts/get":
        case "prompts/list":
          if (!this._capabilities.prompts)
            throw new Error(`Server does not support prompts (required for ${t})`);
          break;
        case "resources/list":
        case "resources/templates/list":
        case "resources/read":
          if (!this._capabilities.resources)
            throw new Error(`Server does not support resources (required for ${t})`);
          break;
        case "tools/call":
        case "tools/list":
          if (!this._capabilities.tools)
            throw new Error(`Server does not support tools (required for ${t})`);
          break;
        case "tasks/get":
        case "tasks/list":
        case "tasks/result":
        case "tasks/cancel":
          if (!this._capabilities.tasks)
            throw new Error(`Server does not support tasks capability (required for ${t})`);
          break;
      }
  }
  assertTaskCapability(t) {
    Lc(this._clientCapabilities?.tasks?.requests, t, "Client");
  }
  assertTaskHandlerCapability(t) {
    this._capabilities && Mc(this._capabilities.tasks?.requests, t, "Server");
  }
  async _oninitialize(t) {
    const n = t.params.protocolVersion;
    return this._clientCapabilities = t.params.capabilities, this._clientVersion = t.params.clientInfo, {
      protocolVersion: Yi.includes(n) ? n : Rn,
      capabilities: this.getCapabilities(),
      serverInfo: this._serverInfo,
      ...this._instructions && { instructions: this._instructions }
    };
  }
  /**
   * After initialization has completed, this will be populated with the client's reported capabilities.
   */
  getClientCapabilities() {
    return this._clientCapabilities;
  }
  /**
   * After initialization has completed, this will be populated with information about the client's name and version.
   */
  getClientVersion() {
    return this._clientVersion;
  }
  getCapabilities() {
    return this._capabilities;
  }
  async ping() {
    return this.request({ method: "ping" }, rt);
  }
  // Implementation
  async createMessage(t, n) {
    if ((t.tools || t.toolChoice) && !this._clientCapabilities?.sampling?.tools)
      throw new Error("Client does not support sampling tools capability.");
    if (t.messages.length > 0) {
      const o = t.messages[t.messages.length - 1], r = Array.isArray(o.content) ? o.content : [o.content], s = r.some((d) => d.type === "tool_result"), i = t.messages.length > 1 ? t.messages[t.messages.length - 2] : void 0, c = i ? Array.isArray(i.content) ? i.content : [i.content] : [], p = c.some((d) => d.type === "tool_use");
      if (s) {
        if (r.some((d) => d.type !== "tool_result"))
          throw new Error("The last message must contain only tool_result content if any is present");
        if (!p)
          throw new Error("tool_result blocks are not matching any tool_use from the previous message");
      }
      if (p) {
        const d = new Set(c.filter((m) => m.type === "tool_use").map((m) => m.id)), f = new Set(r.filter((m) => m.type === "tool_result").map((m) => m.toolUseId));
        if (d.size !== f.size || ![...d].every((m) => f.has(m)))
          throw new Error("ids of tool_result blocks and tool_use blocks from previous message do not match");
      }
    }
    return t.tools ? this.request({ method: "sampling/createMessage", params: t }, Gn, n) : this.request({ method: "sampling/createMessage", params: t }, yt, n);
  }
  /**
   * Creates an elicitation request for the given parameters.
   * For backwards compatibility, `mode` may be omitted for form requests and will default to `'form'`.
   * @param params The parameters for the elicitation request.
   * @param options Optional request options.
   * @returns The result of the elicitation request.
   */
  async elicitInput(t, n) {
    switch (t.mode ?? "form") {
      case "url": {
        if (!this._clientCapabilities?.elicitation?.url)
          throw new Error("Client does not support url elicitation.");
        const r = t;
        return this.request({ method: "elicitation/create", params: r }, je, n);
      }
      case "form": {
        if (!this._clientCapabilities?.elicitation?.form)
          throw new Error("Client does not support form elicitation.");
        const r = t.mode === "form" ? t : { ...t, mode: "form" }, s = await this.request({ method: "elicitation/create", params: r }, je, n);
        if (s.action === "accept" && s.content && r.requestedSchema)
          try {
            const c = this._jsonSchemaValidator.getValidator(r.requestedSchema)(s.content);
            if (!c.valid)
              throw new w(S.InvalidParams, `Elicitation response content does not match requested schema: ${c.errorMessage}`);
          } catch (i) {
            throw i instanceof w ? i : new w(S.InternalError, `Error validating elicitation response: ${i instanceof Error ? i.message : String(i)}`);
          }
        return s;
      }
    }
  }
  /**
   * Creates a reusable callback that, when invoked, will send a `notifications/elicitation/complete`
   * notification for the specified elicitation ID.
   *
   * @param elicitationId The ID of the elicitation to mark as complete.
   * @param options Optional notification options. Useful when the completion notification should be related to a prior request.
   * @returns A function that emits the completion notification when awaited.
   */
  createElicitationCompletionNotifier(t, n) {
    if (!this._clientCapabilities?.elicitation?.url)
      throw new Error("Client does not support URL elicitation (required for notifications/elicitation/complete)");
    return () => this.notification({
      method: "notifications/elicitation/complete",
      params: {
        elicitationId: t
      }
    }, n);
  }
  async listRoots(t, n) {
    return this.request({ method: "roots/list", params: t }, Vn, n);
  }
  /**
   * Sends a logging message to the client, if connected.
   * Note: You only need to send the parameters object, not the entire JSON RPC message
   * @see LoggingMessageNotification
   * @param params
   * @param sessionId optional for stateless and backward compatibility
   */
  async sendLoggingMessage(t, n) {
    if (this._capabilities.logging && !this.isMessageIgnored(t.level, n))
      return this.notification({ method: "notifications/message", params: t });
  }
  async sendResourceUpdated(t) {
    return this.notification({
      method: "notifications/resources/updated",
      params: t
    });
  }
  async sendResourceListChanged() {
    return this.notification({
      method: "notifications/resources/list_changed"
    });
  }
  async sendToolListChanged() {
    return this.notification({ method: "notifications/tools/list_changed" });
  }
  async sendPromptListChanged() {
    return this.notification({ method: "notifications/prompts/list_changed" });
  }
}
class Dc {
  append(t) {
    this._buffer = this._buffer ? Buffer.concat([this._buffer, t]) : t;
  }
  readMessage() {
    if (!this._buffer)
      return null;
    const t = this._buffer.indexOf(`
`);
    if (t === -1)
      return null;
    const n = this._buffer.toString("utf8", 0, t).replace(/\r$/, "");
    return this._buffer = this._buffer.subarray(t + 1), Fc(n);
  }
  clear() {
    this._buffer = void 0;
  }
}
function Fc(e) {
  return ra.parse(JSON.parse(e));
}
function Hc(e) {
  return JSON.stringify(e) + `
`;
}
class Gc {
  constructor(t = $t.stdin, n = $t.stdout) {
    this._stdin = t, this._stdout = n, this._readBuffer = new Dc(), this._started = !1, this._ondata = (o) => {
      this._readBuffer.append(o), this.processReadBuffer();
    }, this._onerror = (o) => {
      this.onerror?.(o);
    };
  }
  /**
   * Starts listening for messages on stdin.
   */
  async start() {
    if (this._started)
      throw new Error("StdioServerTransport already started! If using Server class, note that connect() calls start() automatically.");
    this._started = !0, this._stdin.on("data", this._ondata), this._stdin.on("error", this._onerror);
  }
  processReadBuffer() {
    for (; ; )
      try {
        const t = this._readBuffer.readMessage();
        if (t === null)
          break;
        this.onmessage?.(t);
      } catch (t) {
        this.onerror?.(t);
      }
  }
  async close() {
    this._stdin.off("data", this._ondata), this._stdin.off("error", this._onerror), this._stdin.listenerCount("data") === 0 && this._stdin.pause(), this._readBuffer.clear(), this.onclose?.();
  }
  send(t) {
    return new Promise((n) => {
      const o = Hc(t);
      this._stdout.write(o) ? n() : this._stdout.once("drain", n);
    });
  }
}
const Jn = a.object({
  code: a.string().regex(/^E[0-9]{4}$/),
  severity: a.enum(["error", "warning", "note", "help"]),
  message: a.string(),
  primary_span: a.object({
    file_id: a.number().int().nonnegative(),
    start: a.number().int().nonnegative(),
    end: a.number().int().nonnegative()
  }),
  labels: a.array(a.object({
    file_id: a.number().int().nonnegative(),
    start: a.number().int().nonnegative(),
    end: a.number().int().nonnegative(),
    message: a.string()
  })),
  notes: a.array(a.string()),
  suggested_fixes: a.array(a.object({
    description: a.string(),
    edits: a.array(a.object({
      file_id: a.number().int().nonnegative(),
      start: a.number().int().nonnegative(),
      end: a.number().int().nonnegative(),
      new_text: a.string()
    }))
  })),
  explain_url: a.string().url().optional()
}), Bn = a.object({
  version: a.literal(1),
  functions: a.array(a.object({
    id: a.string(),
    name: a.string(),
    params: a.array(a.object({
      name: a.string(),
      type: a.string(),
      linearity: a.enum(["linear", "unrestricted"])
    })),
    return_type: a.string(),
    effect: a.enum(["pure", "quantum", "prob"]),
    body: a.unknown(),
    spans: a.object({
      file_id: a.number().int().nonnegative(),
      start: a.number().int().nonnegative(),
      end: a.number().int().nonnegative()
    }),
    properties: a.object({
      is_unitary: a.boolean().optional(),
      is_reversible: a.boolean().optional(),
      has_adjoint: a.boolean().optional(),
      has_controlled: a.boolean().optional()
    }).optional()
  })),
  types: a.record(a.object({
    kind: a.enum(["primitive", "qubit", "qubits", "measured", "function", "array", "tuple", "struct", "named"]),
    name: a.string().optional(),
    params: a.array(a.string()).optional(),
    linearity: a.enum(["linear", "unrestricted"]).optional()
  })),
  effects: a.record(a.enum(["pure", "quantum", "prob"])),
  metadata: a.object({
    source_hash: a.string().optional(),
    compiler_version: a.string().optional(),
    timestamp: a.string().datetime().optional()
  }).optional()
}), Vc = a.object({
  property: a.enum(["unitary", "reversible", "uncomputes", "grad_matches", "effect_honesty"]),
  passed: a.boolean(),
  details: a.object({
    function_name: a.string().optional(),
    num_trials: a.number().int().nonnegative().optional(),
    tolerance: a.number().optional(),
    max_error: a.number().optional(),
    counterexample: a.unknown().optional(),
    matrix_norm_diff: a.number().optional(),
    fidelity: a.number().optional()
  })
}), Jc = a.object({
  diagnostics: a.array(Jn),
  gir: Bn.optional()
}), Bc = a.object({
  results: a.array(a.object({
    name: a.string(),
    counts: a.record(a.number().int().nonnegative()),
    probabilities: a.record(a.number())
  })),
  circuit: a.string().optional(),
  shots: a.number().int().nonnegative(),
  seed: a.number().int().nonnegative().optional()
}), Wc = a.object({
  passed: a.boolean(),
  results: a.array(Vc),
  summary: a.object({
    total: a.number().int().nonnegative(),
    passed: a.number().int().nonnegative(),
    failed: a.number().int().nonnegative()
  })
}), Qc = a.object({
  applied: a.array(a.object({
    description: a.string(),
    edits: a.array(a.object({
      file_id: a.number().int().nonnegative(),
      start: a.number().int().nonnegative(),
      end: a.number().int().nonnegative(),
      new_text: a.string()
    }))
  })),
  remaining: a.array(Jn)
}), Kc = a.object({
  code: a.string().regex(/^E[0-9]{4}$/),
  markdown: a.string(),
  examples: a.array(a.object({
    title: a.string(),
    bad: a.string(),
    good: a.string()
  })).optional()
}), Yc = a.object({
  gir: Bn,
  version: a.literal("1")
});
a.object({
  source: a.string(),
  filePath: a.string(),
  position: a.object({
    line: a.number().int().nonnegative(),
    character: a.number().int().nonnegative()
  }),
  queryType: a.enum(["hover", "completion", "definition", "code_lens", "gir_at_position"])
});
const Xc = a.object({
  hover: a.object({
    type: a.string().optional(),
    effect: a.string().optional(),
    linearity: a.string().optional(),
    documentation: a.string().optional(),
    gir_node: a.unknown().optional()
  }).optional(),
  completion: a.array(a.object({
    label: a.string(),
    kind: a.number().optional(),
    detail: a.string().optional(),
    insert_text: a.string().optional()
  })).optional(),
  definition: a.array(a.object({
    file_path: a.string(),
    range: a.object({
      start: a.object({ line: a.number(), character: a.number() }),
      end: a.object({ line: a.number(), character: a.number() })
    })
  })).optional(),
  code_lens: a.array(a.object({
    range: a.object({
      start: a.object({ line: a.number(), character: a.number() }),
      end: a.object({ line: a.number(), character: a.number() })
    }),
    command: a.object({
      title: a.string(),
      command: a.string(),
      arguments: a.array(a.unknown()).optional()
    }).optional()
  })).optional(),
  gir_at_position: a.unknown().optional()
}), eu = Xn(import.meta.url), Jt = Ge(eu, "..");
async function tu() {
  const e = [
    Ge(Jt, "../../../target/release/gala"),
    Ge(Jt, "../../../target/debug/gala"),
    "gala"
  ];
  for (const t of e)
    try {
      const { stdout: n } = await ne(t, ["--version"]);
      if (n.includes("gala")) return t;
    } catch {
    }
  throw new Error("gala binary not found. Run `cargo build --release` or install gala.");
}
let Ie = null;
async function ie() {
  return Ie || (Ie = await tu(), Ie);
}
async function ne(e, t, n = {}) {
  return new Promise((o, r) => {
    const s = Yn(e, t, {
      ...n,
      stdio: ["pipe", "pipe", "pipe"]
    });
    let i = "", c = "";
    s.stdout?.on("data", (d) => {
      i += d.toString();
    }), s.stderr?.on("data", (d) => {
      c += d.toString();
    }), n.input && (s.stdin?.write(n.input), s.stdin?.end());
    const p = setTimeout(() => {
      s.kill("SIGTERM"), r(new Error(`Command timed out after ${n.timeout ?? 3e4}ms`));
    }, n.timeout ?? 3e4);
    s.on("close", (d) => {
      clearTimeout(p), o({ stdout: i, stderr: c, exitCode: d ?? 0 });
    }), s.on("error", (d) => {
      clearTimeout(p), r(d);
    });
  });
}
async function nu(e, t) {
  const n = await ie(), o = ["check", "--json"];
  t && o.push("--input", t);
  const { stdout: r, stderr: s, exitCode: i } = await ne(n, o, {
    input: e
  });
  if (i !== 0 && i !== 1)
    throw new Error(`gala check failed: ${s}`);
  const c = JSON.parse(r);
  return Jc.parse(c);
}
async function Wn(e, t = {}) {
  const n = await ie(), o = ["build", "--json"];
  t.emitGir && (o.push("--emit", "gir=json"), t.girVersion && o.push("--gir-version", String(t.girVersion)));
  const { stdout: r, stderr: s, exitCode: i } = await ne(n, o, { input: e });
  if (i !== 0)
    throw new Error(`gala build failed: ${s}`);
  const c = JSON.parse(r);
  return Yc.parse(c);
}
async function ou(e, t = {}) {
  const n = await ie(), o = ["run", "--json"];
  t.shots && o.push("--shots", String(t.shots)), t.seed && o.push("--seed", String(t.seed));
  const { stdout: r, stderr: s, exitCode: i } = await ne(n, o, { input: e });
  if (i !== 0)
    throw new Error(`gala run failed: ${s}`);
  const c = JSON.parse(r);
  return Bc.parse(c);
}
async function ru(e, t = []) {
  const n = await ie(), o = ["test", "--property", "--json"];
  for (const p of t)
    o.push("--property", p);
  const { stdout: r, stderr: s, exitCode: i } = await ne(n, o, { input: e });
  if (i !== 0 && i !== 1)
    throw new Error(`gala test failed: ${s}`);
  const c = JSON.parse(r);
  return Wc.parse(c);
}
async function su(e) {
  const t = await ie(), n = ["fix", "--json"], { stdout: o, stderr: r, exitCode: s } = await ne(t, n, { input: e });
  if (s !== 0)
    throw new Error(`gala fix failed: ${r}`);
  const i = JSON.parse(o);
  return Qc.parse(i);
}
async function iu(e) {
  const t = await ie(), n = ["explain", e, "--markdown"], { stdout: o, stderr: r, exitCode: s } = await ne(t, n);
  if (s !== 0)
    throw new Error(`gala explain failed: ${r}`);
  const i = JSON.parse(o);
  return Kc.parse(i);
}
async function au(e) {
  return Wn(e, { emitGir: !0, girVersion: 1 });
}
async function cu(e) {
  return Xc.parse({
    hover: void 0,
    completion: void 0,
    definition: void 0,
    code_lens: void 0,
    gir_at_position: void 0
  });
}
const oe = new Uc(
  { name: "gala-mcp", version: "0.1.0" },
  { capabilities: { tools: {}, resources: {}, prompts: {} } }
), B = {
  gala_check: a.object({
    source: a.string(),
    filePath: a.string().optional()
  }),
  gala_build: a.object({
    source: a.string(),
    emitGir: a.boolean().optional(),
    girVersion: a.number().int().positive().optional()
  }),
  gala_run: a.object({
    source: a.string(),
    shots: a.number().int().positive().optional(),
    seed: a.number().int().nonnegative().optional()
  }),
  gala_test: a.object({
    source: a.string(),
    properties: a.array(a.enum(["unitary", "reversible", "uncomputes", "grad_matches", "effect_honesty"])).optional()
  }),
  gala_fix: a.object({
    source: a.string()
  }),
  gala_explain: a.object({
    code: a.string().regex(/^E[0-9]{4}$/)
  }),
  gala_gir: a.object({
    source: a.string()
  }),
  gala_lsp_query: a.object({
    source: a.string(),
    filePath: a.string(),
    position: a.object({
      line: a.number().int().nonnegative(),
      character: a.number().int().nonnegative()
    }),
    queryType: a.enum(["hover", "completion", "definition", "code_lens", "gir_at_position"])
  })
};
oe.setRequestHandler(Fn, async () => ({
  tools: [
    {
      name: "gala_check",
      description: "Type-check Gala source code and return structured diagnostics with error codes, spans, and suggested fixes",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code to check" },
          file_path: { type: "string", description: "Optional file path for error reporting" }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_build",
      description: "Build Gala source and optionally emit GIR (Gala Intermediate Representation)",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code" },
          emit_gir: { type: "boolean", description: "Whether to emit GIR JSON" },
          gir_version: { type: "number", description: "GIR schema version (default: 1)" }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_run",
      description: "Execute Gala program on the built-in quantum simulator",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code" },
          shots: { type: "number", description: "Number of measurement shots (default: 1024)" },
          seed: { type: "number", description: "Random seed for reproducibility" }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_test",
      description: "Run quantum property tests (unitarity, reversibility, uncomputation, gradient correctness)",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code with #[property(...)] attributes" },
          properties: {
            type: "array",
            items: { type: "string", enum: ["unitary", "reversible", "uncomputes", "grad_matches", "effect_honesty"] },
            description: "Specific properties to test (default: all)"
          }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_fix",
      description: "Apply suggested fixes for diagnostics in Gala source code",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code to fix" }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_explain",
      description: "Get detailed explanation of a Gala diagnostic error code",
      inputSchema: {
        type: "object",
        properties: {
          code: { type: "string", pattern: "^E[0-9]{4}$", description: "Error code (e.g., E0412)" }
        },
        required: ["code"]
      }
    },
    {
      name: "gala_gir",
      description: "Get Gala IR (GIR) for semantic analysis of quantum/classical code",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code" }
        },
        required: ["source"]
      }
    },
    {
      name: "gala_lsp_query",
      description: "Query the Gala Language Server for semantic information (hover, completion, GIR at position)",
      inputSchema: {
        type: "object",
        properties: {
          source: { type: "string", description: "Gala source code" },
          file_path: { type: "string", description: "File path for LSP context" },
          position: {
            type: "object",
            properties: {
              line: { type: "number", minimum: 0 },
              character: { type: "number", minimum: 0 }
            },
            required: ["line", "character"]
          },
          query_type: {
            type: "string",
            enum: ["hover", "completion", "definition", "code_lens", "gir_at_position"]
          }
        },
        required: ["source", "file_path", "position", "query_type"]
      }
    }
  ]
}));
oe.setRequestHandler(kt, async (e) => {
  const { name: t, arguments: n } = e.params;
  try {
    let o;
    switch (t) {
      case "gala_check": {
        const r = B.gala_check.parse(n);
        o = await nu(r.source, r.filePath);
        break;
      }
      case "gala_build": {
        const r = B.gala_build.parse(n);
        o = await Wn(r.source, { emitGir: r.emitGir, girVersion: r.girVersion });
        break;
      }
      case "gala_run": {
        const r = B.gala_run.parse(n);
        o = await ou(r.source, { shots: r.shots, seed: r.seed });
        break;
      }
      case "gala_test": {
        const r = B.gala_test.parse(n);
        o = await ru(r.source, r.properties);
        break;
      }
      case "gala_fix": {
        const r = B.gala_fix.parse(n);
        o = await su(r.source);
        break;
      }
      case "gala_explain": {
        const r = B.gala_explain.parse(n);
        o = await iu(r.code);
        break;
      }
      case "gala_gir": {
        const r = B.gala_gir.parse(n);
        o = await au(r.source);
        break;
      }
      case "gala_lsp_query": {
        const r = B.gala_lsp_query.parse(n);
        o = await cu(r);
        break;
      }
      default:
        throw new Error(`Unknown tool: ${t}`);
    }
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(o, null, 2)
        }
      ]
    };
  } catch (o) {
    const r = o instanceof Error ? o.message : String(o);
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify({ error: r }, null, 2)
        }
      ],
      isError: !0
    };
  }
});
a.object({
  file: a.string()
}), a.object({
  file: a.string()
}), a.object({}), a.object({});
oe.setRequestHandler(An, async () => ({
  resources: [
    {
      uri: "gala://schema/diagnostic.v1",
      name: "Diagnostic JSON Schema v1",
      mimeType: "application/json",
      description: "JSON schema for Gala structured diagnostics"
    },
    {
      uri: "gala://schema/gir.v1",
      name: "GIR JSON Schema v1",
      mimeType: "application/json",
      description: "JSON schema for Gala Intermediate Representation"
    },
    {
      uri: "gala://schema/property_result.v1",
      name: "Property Test Result JSON Schema v1",
      mimeType: "application/json",
      description: "JSON schema for quantum property test results"
    }
  ]
}));
oe.setRequestHandler(Mn, async (e) => {
  const { uri: t } = e.params;
  if (t === "gala://schema/diagnostic.v1") {
    const n = await import("./diagnostic.v1-Cc33rZMO.js");
    return { contents: [{ uri: t, mimeType: "application/json", text: JSON.stringify(n.default, null, 2) }] };
  }
  if (t === "gala://schema/gir.v1") {
    const n = await import("./gir.v1-BfC1dnqK.js");
    return { contents: [{ uri: t, mimeType: "application/json", text: JSON.stringify(n.default, null, 2) }] };
  }
  if (t === "gala://schema/property_result.v1") {
    const n = await import("./property_result.v1-s3ZRD4zs.js");
    return { contents: [{ uri: t, mimeType: "application/json", text: JSON.stringify(n.default, null, 2) }] };
  }
  throw new Error(`Resource not found: ${t}`);
});
oe.setRequestHandler(Ln, async () => ({
  prompts: [
    {
      name: "gala/debug-error",
      description: "Given a Gala diagnostic, explain the quantum physics and suggest fixes",
      arguments: [
        {
          name: "diagnostic",
          description: "JSON diagnostic object from gala_check",
          required: !0
        }
      ]
    },
    {
      name: "gala/write-quantum-fn",
      description: "Write a Gala quantum function implementing a specific algorithm",
      arguments: [
        {
          name: "task",
          description: "Description of the quantum algorithm (e.g., 'Bell pair preparation', 'QFT on 4 qubits')",
          required: !0
        },
        {
          name: "constraints",
          description: "Optional constraints (e.g., 'use only H, CX, Rz gates', 'no mid-circuit measurement')",
          required: !1
        }
      ]
    },
    {
      name: "gala/optimize-circuit",
      description: "Optimize a Gala quantum function for a specific backend",
      arguments: [
        {
          name: "source",
          description: "Gala source code of the quantum function",
          required: !0
        },
        {
          name: "backend",
          description: "Target backend (e.g., 'ibm_brisbane', 'ionq_aria', 'simulator')",
          required: !0
        },
        {
          name: "objectives",
          description: "Optimization objectives (e.g., 'minimize depth', 'minimize 2q gates', 'maximize fidelity')",
          required: !1
        }
      ]
    }
  ]
}));
oe.setRequestHandler(Un, async (e) => {
  const { name: t, arguments: n } = e.params;
  switch (t) {
    case "gala/debug-error": {
      const o = n?.diagnostic;
      return {
        description: "Debug a Gala diagnostic error",
        messages: [
          {
            role: "user",
            content: {
              type: "text",
              text: `Analyze this Gala diagnostic and explain the quantum physics behind the error, then suggest fixes:

${JSON.stringify(o, null, 2)}`
            }
          }
        ]
      };
    }
    case "gala/write-quantum-fn": {
      const o = n?.task, r = n?.constraints ?? "";
      return {
        description: "Write a Gala quantum function",
        messages: [
          {
            role: "user",
            content: {
              type: "text",
              text: `Write a Gala quantum function that implements: ${o}

Constraints: ${r}

Requirements:
- Use proper Gala syntax with effect annotations (quantum/prob/pure)
- Include linear type annotations for qubits
- Add #[property(unitary)] for verification
- Use standard library gates from gala.gates`
            }
          }
        ]
      };
    }
    case "gala/optimize-circuit": {
      const o = n?.source, r = n?.backend, s = n?.objectives ?? "minimize depth and 2-qubit gate count";
      return {
        description: "Optimize a Gala quantum circuit for a backend",
        messages: [
          {
            role: "user",
            content: {
              type: "text",
              text: `Optimize this Gala quantum function for ${r}:

${o}

Objectives: ${s}

Provide optimized Gala code with #[property(unitary)] and explain the optimizations applied.`
            }
          }
        ]
      };
    }
    default:
      throw new Error(`Unknown prompt: ${t}`);
  }
});
async function uu() {
  const e = new Gc();
  await oe.connect(e), console.error("Gala MCP server running on stdio");
}
uu().catch((e) => {
  console.error("Server error:", e), process.exit(1);
});
