# ADR 0017: Explicit AI cancellation

Status: accepted and implemented locally.

A bounded native request registry owns one-use input previews and one running provider request. Generate claims its preview before worker dispatch; Stop can cancel a claimed token or consume an unclaimed preview, so either command arrival order prevents accidental reuse. A running guard releases the slot on success, error or early return. Cancellation does not depend on the Git operation mutex.

The process runner accepts an optional cooperative cancellation token, preserving ordinary Git behavior. AI version probes and inference subprocesses check it before execution, during bounded polling and before returning output. Cancellation uses the existing process-group/tree termination and joins stdin/stdout/stderr workers. Codex cancellation remains distinguishable from authentication errors. No automatic retry occurs.

Ollama generation uses an async reqwest request on a short-lived current-thread Tokio runtime inside the blocking worker. Fifty-millisecond cooperative checks allow Stop to drop the pending future and runtime, closing local HTTP work while waiting for headers or body. It retains fixed loopback origin, no proxies/redirects, a 120-second request deadline and 256 KiB response cap. The existing model-discovery request stays separately bounded at three seconds. Tokio was already a locked transitive dependency; it is now declared directly with runtime/time features. No new provider SDK or service is introduced.

`AiRequest` and `useAiRequest` own the renderer lifecycle for both ordinary and merge assistance. Stop immediately suppresses late answers, while busy/Stopping stays until the original worker resolves; failed cancellation delivery allows an explicit Stop retry. Completion/error clears consumed UI preview state. A new generation requires a fresh prepared input. Manual drafts/buffers are not replaced, and generated results still require explicit insertion. Repository fingerprint checks still precede sending and follow generation.

Stop terminates local transport/process work; a remote provider or local model server may already have processed/billed the request and may continue server-side computation. Quitting/crashing the app is not equivalent to a verified Stop, and automatic crash recovery of in-flight jobs remains unimplemented. Windows task-tree termination awaits actual platform runtime verification. Offline subprocess and loopback fixtures test the mechanisms without new live inference.
