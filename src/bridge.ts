import { invoke, isTauri } from "@tauri-apps/api/core";
export const desktop = isTauri();
export function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!desktop)
    return Promise.reject(
      new Error(
        "Open the installed Ramo application to access Git. This web view does not expose your filesystem.",
      ),
    );
  return invoke<T>(command, args);
}
