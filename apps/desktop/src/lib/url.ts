/** Host names that always mean "this computer". */
const LOOPBACK_HOSTS = ["localhost", "127.0.0.1", "[::1]"];

function parse(address: string): URL | null {
  try {
    return new URL(address.trim());
  } catch {
    return null;
  }
}

/** The host name of a URL, or `null` if it is not a URL. */
export function hostOf(address: string): string | null {
  return parse(address)?.hostname ?? null;
}

/** Whether a URL points at this computer. */
export function isLoopbackUrl(address: string): boolean {
  const url = parse(address);
  return url !== null && LOOPBACK_HOSTS.includes(url.hostname);
}

/**
 * Whether an address may be used for a service that receives text and an
 * API key: `https`, or `http` when the service is on this computer. The
 * core applies the same rule; this copy only lets the UI say so early.
 */
export function isAcceptableServiceUrl(address: string): boolean {
  const url = parse(address);
  if (url === null) {
    return false;
  }
  return url.protocol === "https:" || (url.protocol === "http:" && isLoopbackUrl(address));
}
