import {
  Archive,
  ArrowUUpLeft,
  DotsThree,
  GitBranch,
} from "@phosphor-icons/react";
import type { MouseEvent } from "react";
import { isConflicted, type Snapshot } from "../domain";

/** Shortcuts open the same reviewed operations as the sidebar and menus. */
export function LocalActions({
  snapshot,
  locked,
  branch,
  stash,
  pop,
  more,
}: {
  snapshot: Snapshot;
  locked: boolean;
  branch: () => void;
  stash: () => void;
  pop: () => void;
  more: (event: MouseEvent<HTMLButtonElement>) => void;
}) {
  const blocked =
    locked || !!snapshot.operation || snapshot.files.some(isConflicted);
  return (
    <div className="local-actions" role="group" aria-label="Local Git actions">
      <button
        disabled={blocked}
        onClick={branch}
        title="Create and switch to a new branch"
      >
        <GitBranch size={18} />
        <span>Branch</span>
      </button>
      <button
        disabled={blocked || !snapshot.head || !snapshot.files.length}
        onClick={stash}
        title="Save working changes in a stash"
      >
        <Archive size={18} />
        <span>Stash</span>
      </button>
      <button
        disabled={blocked || !snapshot.stashes.length}
        onClick={pop}
        title="Review the latest stash before restoring and removing it"
      >
        <ArrowUUpLeft size={18} />
        <span>Pop</span>
      </button>
      <button
        className="icon"
        disabled={locked}
        onClick={more}
        aria-label="More repository actions"
        aria-haspopup="menu"
        title="More repository actions"
      >
        <DotsThree size={22} />
      </button>
    </div>
  );
}
