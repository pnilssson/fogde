export type Status = "connected" | "connecting" | "disconnected";

export function statusOf(connected: boolean, busy: boolean): Status {
  if (connected) {
    return "connected";
  }

  return busy ? "connecting" : "disconnected";
}
