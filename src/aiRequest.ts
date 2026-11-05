import type { AiConfig } from "./domain";
export interface AiRequestState {
  pending: boolean;
  stopping: boolean;
  notice: string;
}
export const stoppedNotice =
  "AI result discarded after Stop. Your draft is unchanged. Provider usage may already have occurred.";
/** One request per owner. Stop suppresses even a result that beats native cancellation. */
export class AiRequest {
  private active: { id: string; stopped: boolean } | null = null;
  state: AiRequestState = { pending: false, stopping: false, notice: "" };
  constructor(
    private transport: {
      generate: (id: string, config: AiConfig) => Promise<string>;
      cancel: (id: string) => Promise<void>;
    },
    private changed: (state: AiRequestState) => void,
  ) {}
  private publish(next: AiRequestState) {
    this.state = next;
    this.changed(next);
  }
  clearNotice() {
    if (!this.active) this.publish({ ...this.state, notice: "" });
  }
  async run(id: string, config: AiConfig): Promise<string | null> {
    if (this.active) throw new Error("An AI request is still finishing.");
    const request = { id, stopped: false };
    this.active = request;
    this.publish({ pending: true, stopping: false, notice: "" });
    try {
      const answer = await this.transport.generate(id, config);
      return request.stopped ? null : answer;
    } catch (error) {
      if (!request.stopped) throw error;
      return null;
    } finally {
      this.active = null;
      this.publish({
        pending: false,
        stopping: false,
        notice: request.stopped ? stoppedNotice : "",
      });
    }
  }
  async stop(): Promise<void> {
    const request = this.active;
    if (!request || this.state.stopping) return;
    request.stopped = true;
    this.publish({
      pending: true,
      stopping: true,
      notice:
        "Stopping the local request… Provider usage may already have occurred.",
    });
    try {
      await this.transport.cancel(request.id);
    } catch (error) {
      if (this.active === request)
        this.publish({
          pending: true,
          stopping: false,
          notice: `Could not confirm Stop. Waiting for the provider; try Stop again. ${String(error)}`,
        });
    }
  }
}
