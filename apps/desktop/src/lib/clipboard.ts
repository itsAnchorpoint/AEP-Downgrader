/** Puts text on the clipboard. The async API needs a secure context; the fallback covers older webviews. */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    return;
  } catch {
    // fall through to the selection-based copy
  }
  const area = document.createElement("textarea");
  area.value = text;
  area.setAttribute("readonly", "");
  area.style.cssText = "position:fixed;top:0;left:0;opacity:0;pointer-events:none";
  document.body.append(area);
  area.select();
  try {
    if (!document.execCommand("copy")) throw new Error("copy command refused");
  } finally {
    area.remove();
  }
}
