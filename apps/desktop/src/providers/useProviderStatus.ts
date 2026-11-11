import { useCallback, useEffect, useState } from "react";

export interface ProviderStatusState<Status> {
  /** `null` until loaded, and when the core cannot be reached (for example in a plain browser). */
  provider: Status | null;
  hasLoaded: boolean;
  /** Fetches the status again, for example after the API key was saved or removed. */
  reload: () => Promise<void>;
}

/**
 * Loads a provider's description and key status from the core.
 * `fetchStatus` must be a stable function, such as a module-level import.
 */
export function useProviderStatus<Status>(
  fetchStatus: () => Promise<Status>,
): ProviderStatusState<Status> {
  const [provider, setProvider] = useState<Status | null>(null);
  const [hasLoaded, setHasLoaded] = useState(false);

  const fetchOrNull = useCallback(() => fetchStatus().catch(() => null), [fetchStatus]);

  useEffect(() => {
    let isCurrent = true;
    void fetchOrNull().then((fetched) => {
      if (isCurrent) {
        setProvider(fetched);
        setHasLoaded(true);
      }
    });
    return () => {
      isCurrent = false;
    };
  }, [fetchOrNull]);

  const reload = useCallback(async () => {
    setProvider(await fetchOrNull());
    setHasLoaded(true);
  }, [fetchOrNull]);

  return { provider, hasLoaded, reload };
}
