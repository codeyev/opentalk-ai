import { invoke } from "@tauri-apps/api/core";

/** Mirrors the `AppInfo` struct returned by the Rust `app_info` command. */
export interface AppInfo {
  name: string;
  version: string;
}

function isAppInfo(value: unknown): value is AppInfo {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return typeof candidate.name === "string" && typeof candidate.version === "string";
}

/** Asks the Rust core for its name and version. Rejects if the reply is malformed. */
export async function fetchAppInfo(): Promise<AppInfo> {
  const reply: unknown = await invoke("app_info");
  if (!isAppInfo(reply)) {
    throw new Error("Unexpected response from the application core.");
  }
  return reply;
}
