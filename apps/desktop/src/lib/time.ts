export function short(elapsed: number) {
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 1) {
    return "now";
  }
  if (minutes < 60) {
    return `${String(minutes)}m`;
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${String(hours)}h`;
  }

  const days = Math.floor(hours / 24);
  if (days < 7) {
    return `${String(days)}d`;
  }
  if (days < 30) {
    return `${String(Math.floor(days / 7))}w`;
  }

  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${String(months)}mo`;
  }

  return `${String(Math.max(1, Math.floor(days / 365)))}y`;
}

export function ago(time: string | number, now = Date.now()) {
  const at = typeof time === "number" ? time : Date.parse(time);
  const elapsed = short(now - at);

  return elapsed === "now" ? "just now" : `${elapsed} ago`;
}

export function countdown(milliseconds: number) {
  const minutes = Math.max(1, Math.ceil(milliseconds / 60_000));
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);

  if (days > 0) {
    return `${String(days)}d ${String(hours)}h`;
  }
  if (hours > 0) {
    return `${String(hours)}h ${String(minutes % 60)}m`;
  }

  return `${String(minutes)}m`;
}
