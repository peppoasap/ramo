import { describe, expect, it } from "vitest";
import { AiRequest, stoppedNotice } from "./aiRequest";
const config = { provider: "synthetic", model: "fixture" };
function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: unknown) => void;
  const promise = new Promise<T>((a, b) => {
    resolve = a;
    reject = b;
  });
  return { promise, resolve, reject };
}
describe("AI request cancellation", () => {
  it("discards a response that wins the native Stop race and preserves the draft", async () => {
    const reply = deferred<string>(),
      cancel = deferred<void>(),
      next = deferred<string>();
    const ids: string[] = [];
    const request = new AiRequest(
      {
        generate: async (id) => (id === "one" ? reply.promise : next.promise),
        cancel: async (id) => {
          ids.push(id);
          return cancel.promise;
        },
      },
      () => {},
    );
    const manual = "Manual draft\nwith Unicode è";
    let draft = manual;
    const running = request.run("one", config);
    const stop = request.stop();
    reply.resolve("Late AI message");
    const answer = await running;
    if (answer !== null) draft = answer;
    expect(draft).toBe(manual);
    expect(request.state.pending).toBe(false);
    expect(request.state.notice).toBe(stoppedNotice);
    expect(ids).toEqual(["one"]);
    const nextRun = request.run("two", config);
    cancel.reject("late cancellation delivery");
    await stop;
    expect(request.state.pending).toBe(true);
    expect(request.state.notice).toBe("");
    next.resolve("fresh");
    expect(await nextRun).toBe("fresh");
  });
  it("waits for transport completion and refuses overlapping generation", async () => {
    const reply = deferred<string>();
    const request = new AiRequest(
      { generate: async () => reply.promise, cancel: async () => {} },
      () => {},
    );
    const running = request.run("one", config);
    await request.stop();
    expect(request.state).toMatchObject({ pending: true, stopping: true });
    const stoppingNotice = request.state.notice;
    request.clearNotice();
    expect(request.state.notice).toBe(stoppingNotice);
    await expect(request.run("two", config)).rejects.toThrow("still finishing");
    reply.reject("native cancelled");
    expect(await running).toBeNull();
    expect(request.state.pending).toBe(false);
    request.clearNotice();
    expect(request.state.notice).toBe("");
  });
  it("allows Stop retry after IPC failure while suppressing late output", async () => {
    const reply = deferred<string>();
    let calls = 0;
    const request = new AiRequest(
      {
        generate: async () => reply.promise,
        cancel: async () => {
          if (++calls === 1) throw new Error("IPC unavailable");
        },
      },
      () => {},
    );
    const running = request.run("one", config);
    await request.stop();
    expect(request.state).toMatchObject({ pending: true, stopping: false });
    expect(request.state.notice).toContain("Could not confirm");
    await request.stop();
    expect(calls).toBe(2);
    reply.resolve("discard");
    expect(await running).toBeNull();
  });
  it("returns normal results and errors without changing them, then permits a fresh request", async () => {
    let calls = 0;
    const request = new AiRequest(
      {
        generate: async () => {
          if (++calls === 1) throw new Error("Provider failed");
          return "Fresh message";
        },
        cancel: async () => {},
      },
      () => {},
    );
    await expect(request.run("one", config)).rejects.toThrow("Provider failed");
    expect(request.state.pending).toBe(false);
    expect(await request.run("two", config)).toBe("Fresh message");
  });
});
