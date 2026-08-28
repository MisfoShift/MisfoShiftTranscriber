export function appendTranscript(current: string, incoming: string): string {
  const next = incoming.trim();
  if (!next) return current;
  const previous = current.trimEnd();
  return previous ? `${previous}\n${next}` : next;
}

export function transcriptFileName(now = new Date()): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  const stamp = [
    now.getFullYear(),
    pad(now.getMonth() + 1),
    pad(now.getDate()),
    "-",
    pad(now.getHours()),
    pad(now.getMinutes()),
  ].join("");
  return `transcript-${stamp}.txt`;
}
