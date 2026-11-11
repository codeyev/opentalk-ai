import { getTranscriptionProvider } from "./providerClient";
import { useProviderStatus } from "./useProviderStatus";

/** The transcription provider's description and key status. */
export function useTranscriptionProvider() {
  return useProviderStatus(getTranscriptionProvider);
}
