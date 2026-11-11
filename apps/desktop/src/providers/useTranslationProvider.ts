import { getTranslationProvider } from "./providerClient";
import { useProviderStatus } from "./useProviderStatus";

/** The translation provider's description and key status. */
export function useTranslationProvider() {
  return useProviderStatus(getTranslationProvider);
}
