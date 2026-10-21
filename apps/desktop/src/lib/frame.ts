import { invoke } from "@tauri-apps/api/core";

import type { Frame } from "@/generated/protocol";

export const frame = await invoke<Frame>("frame");
