import type { Commit } from "./domain";
export interface Edge {
  from: number;
  to: number;
  color: number;
  fromNode: boolean;
}
export interface GraphRow {
  incoming: boolean;
  lane: number;
  lanes: number;
  color: number;
  edges: Edge[];
}
/** Stable topological lanes, derived exclusively from parent OIDs, never commit dates or labels. */
export function layoutGraph(commits: Commit[]): GraphRow[] {
  let lanes: (string | null)[] = [];
  return commits.map((commit) => {
    let lane = lanes.indexOf(commit.oid);
    const incoming = lane >= 0;
    if (lane < 0) {
      lane = lanes.indexOf(null);
      if (lane < 0) lane = lanes.length;
      lanes[lane] = commit.oid;
    }
    const before = lanes.slice();
    lanes[lane] = null;
    const targets: number[] = [];
    for (let index = 0; index < commit.parents.length; index++) {
      const parent = commit.parents[index];
      let target = lanes.indexOf(parent);
      if (target < 0) {
        target =
          index === 0 && lanes[lane] === null ? lane : lanes.indexOf(null);
        if (target < 0) target = lanes.length;
        lanes[target] = parent;
      }
      targets.push(target);
    }
    const edges: Edge[] = [];
    before.forEach((hash, from) => {
      if (hash && from !== lane)
        edges.push({ from, to: from, color: from, fromNode: false });
    });
    targets.forEach((to) =>
      edges.push({ from: lane, to, color: to, fromNode: true }),
    );
    const width = Math.max(before.length, lanes.length, lane + 1);
    while (lanes.length && lanes[lanes.length - 1] === null) lanes.pop();
    return { incoming, lane, lanes: width, color: lane, edges };
  });
}
