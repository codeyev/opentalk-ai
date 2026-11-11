import { getReplyProvider } from "./providerClient";
import { useProviderStatus } from "./useProviderStatus";

/** The reply provider's description and key status. */
export function useReplyProvider() {
  return useProviderStatus(getReplyProvider);
}
