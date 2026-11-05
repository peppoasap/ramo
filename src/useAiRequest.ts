import { useRef, useState } from "react";
import { call } from "./bridge";
import { AiRequest, type AiRequestState } from "./aiRequest";
export function useAiRequest() {
  const [state, setState] = useState<AiRequestState>({
    pending: false,
    stopping: false,
    notice: "",
  });
  const request = useRef<AiRequest | null>(null);
  if (!request.current)
    request.current = new AiRequest(
      {
        generate: (inputId, config) =>
          call<string>("generate_ai", { inputId, config }),
        cancel: (inputId) => call<void>("cancel_ai", { inputId }),
      },
      setState,
    );
  return {
    ...state,
    run: request.current.run.bind(request.current),
    stop: request.current.stop.bind(request.current),
    clearNotice: request.current.clearNotice.bind(request.current),
  };
}
