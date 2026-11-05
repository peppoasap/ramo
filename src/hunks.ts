import type { FileChange } from "./domain";
import { isConflicted } from "./domain";
export function supportsHunks(
  file: FileChange | undefined,
  text: string,
  unstage: boolean,
): boolean {
  if (
    !file ||
    isConflicted(file) ||
    file.original ||
    (unstage ? file.index !== "M" : file.worktree !== "M" || file.index === "?")
  )
    return false;
  if (
    text.length > 256 * 1024 ||
    new TextEncoder().encode(text).length > 256 * 1024
  )
    return false;
  if (
    /^(diff --cc |diff --combined |@@@|GIT binary patch|Binary files |old mode |new mode |new file mode |deleted file mode |rename from |rename to |copy from |copy to )/m.test(
      text,
    )
  )
    return false;
  if (!/^index [a-f0-9]+\.\.[a-f0-9]+ (100644|100755)$/m.test(text))
    return false;
  return /^@@ -\d+(?:,\d+)? \+\d+(?:,\d+)? @@/m.test(text);
}
export async function reviewedPatchHash(text: string): Promise<string> {
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(text),
  );
  return Array.from(new Uint8Array(bytes), (value) =>
    value.toString(16).padStart(2, "0"),
  ).join("");
}
