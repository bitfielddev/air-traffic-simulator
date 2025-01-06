export function escape(text: string) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("'", "&#39;")
    .replaceAll('"', "&quot;");
}

// https://stackoverflow.com/questions/6312993/javascript-seconds-to-time-string-with-format-hhmmss
export function formatDuration(d: number): string {
  let h: string | number = Math.floor(d / 3600);
  let m: string | number = Math.floor((d % 3600) / 60);
  let s: string | number = Math.floor(d % 60);

  if (h < 10) {
    h = "0" + h;
  }
  if (m < 10) {
    m = "0" + m;
  }
  if (s < 10) {
    s = "0" + s;
  }
  return h + ":" + m + ":" + s;
}
