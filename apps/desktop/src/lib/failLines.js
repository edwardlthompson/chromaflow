/** Split long helper errors into a first line plus optional details. */

export function failLines(msg) {
  const text = String(msg || "").trim();
  if (!text) return { first: "", rest: "" };
  const nl = text.indexOf("\n");
  if (nl > 0) return { first: text.slice(0, nl).trim(), rest: text.slice(nl + 1).trim() };
  const cut = text.search(/[.!?](?:\s|$)/);
  if (cut >= 0 && cut < 120) {
    return { first: text.slice(0, cut + 1).trim(), rest: text.slice(cut + 1).trim() };
  }
  if (text.length <= 120) return { first: text, rest: "" };
  return { first: `${text.slice(0, 120).trim()}…`, rest: text };
}
