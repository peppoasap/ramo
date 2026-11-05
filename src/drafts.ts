// Only commit text is written here; provider auth and diff input are not persisted.
type Storage = Pick<globalThis.Storage, "getItem" | "setItem" | "removeItem">;
const prefix = "ramo.commit-draft.v1.";
const maximumLength = 32768;

function key(repoId: string) {
  if (!/^[a-f0-9]{64}$/.test(repoId))
    throw new Error("Invalid repository identity");
  return prefix + repoId;
}
export function loadDraft(storage: Storage, repoId: string): string {
  const text = storage.getItem(key(repoId)) ?? "";
  if (text.length > maximumLength) throw new Error("Saved draft is too large");
  return text;
}
export function saveDraft(storage: Storage, repoId: string, text: string) {
  if (text.length > maximumLength) throw new Error("Commit draft is too large");
  if (text) storage.setItem(key(repoId), text);
  else storage.removeItem(key(repoId));
}
// A completed commit must not erase a newer draft saved while Git was running.
export function clearCommittedDraft(
  storage: Storage,
  repoId: string,
  submitted: string,
) {
  if (loadDraft(storage, repoId) === submitted) storage.removeItem(key(repoId));
}
