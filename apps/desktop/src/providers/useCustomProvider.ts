import { useCallback } from "react";
import { getCustomProvider } from "./providerClient";
import { useProviderStatus } from "./useProviderStatus";

/**
 * Whether an API key is saved for the OpenAI-compatible service at
 * `baseUrl`. Each service has a key of its own, so the answer changes
 * with the address.
 */
export function useCustomProvider(baseUrl: string) {
  const fetchStatus = useCallback(() => getCustomProvider(baseUrl), [baseUrl]);
  return useProviderStatus(fetchStatus);
}
