/** Turn lighting backend detail into a short status line. */

export function applyStatus(detail, okText) {
  const text = String(detail || "").trim();
  if (!text) return okText;
  const extra = /^broadcast [0-9A-Fa-f]+(?: \((.+)\))?$/i.exec(text);
  if (extra) return extra[1] || okText;
  return okText;
}
