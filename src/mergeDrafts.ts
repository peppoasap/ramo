/** Local recovery for edited resolution buffers. Provider inputs/source versions are not stored. */
type Storage = Pick<globalThis.Storage, "getItem" | "setItem" | "removeItem">;
export interface MergeDraft {
  version: 1;
  fingerprint: string;
  content: string;
  remove: boolean;
  updatedAt: number;
}
const maxBytes = 1024 * 1024;
export function allowsMergeRecovery(path: string): boolean {
  return !path
    .toLowerCase()
    .split(/[\\/]/)
    .some(
      (part) =>
        part === ".env" ||
        part.startsWith(".env.") ||
        ["credentials", "credentials.json", "id_rsa", "id_ed25519"].includes(
          part,
        ) ||
        /\.(pem|key|p12)$/.test(part),
    );
}
export async function mergeDraftKey(
  repoId: string,
  path: string,
): Promise<string> {
  if (
    !/^[a-f0-9]{64}$/.test(repoId) ||
    !path ||
    path.includes("\0") ||
    path.length > 4096
  )
    throw new Error("Invalid merge recovery identity");
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(path),
  );
  const hash = Array.from(new Uint8Array(bytes), (value) =>
    value.toString(16).padStart(2, "0"),
  ).join("");
  return `ramo.merge-draft.v1.${repoId}.${hash}`;
}
function validate(value: unknown): MergeDraft {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Invalid saved resolution");
  const draft = value as Partial<MergeDraft>;
  if (
    draft.version !== 1 ||
    typeof draft.fingerprint !== "string" ||
    !/^[a-f0-9]{64}$/.test(draft.fingerprint) ||
    typeof draft.content !== "string" ||
    draft.content.includes("\0") ||
    new TextEncoder().encode(draft.content).length > maxBytes ||
    typeof draft.remove !== "boolean" ||
    typeof draft.updatedAt !== "number" ||
    !Number.isFinite(draft.updatedAt) ||
    draft.updatedAt < 0
  )
    throw new Error("Invalid or oversized saved resolution");
  return draft as MergeDraft;
}
/** Compare before writing/clearing so a detected newer window's draft is preserved. */
export class MergeDraftSession {
  private previous: string | null = null;
  private loaded = false;
  constructor(
    private storage: Storage,
    private key: string,
  ) {}
  load(): MergeDraft | null {
    const raw = this.storage.getItem(this.key);
    if (raw && raw.length > maxBytes * 6 + 1024)
      throw new Error("Saved resolution is too large");
    const value = raw === null ? null : validate(JSON.parse(raw));
    this.previous = raw;
    this.loaded = true;
    return value;
  }
  private check() {
    if (!this.loaded) throw new Error("Recovery is not ready");
    if (this.storage.getItem(this.key) !== this.previous)
      throw new Error(
        "Another window changed this saved resolution. Copy your current result before reopening.",
      );
  }
  save(fingerprint: string, content: string, remove: boolean) {
    this.check();
    const draft = validate({
      version: 1,
      fingerprint,
      content,
      remove,
      updatedAt: Date.now(),
    });
    const raw = JSON.stringify(draft);
    this.storage.setItem(this.key, raw);
    this.previous = raw;
  }
  clear() {
    this.check();
    this.storage.removeItem(this.key);
    this.previous = null;
  }
}
