export function trusted(url: string, domains: readonly string[]) {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }

  return (
    parsed.protocol === "https:" &&
    domains.some(
      (domain) =>
        parsed.hostname === domain || parsed.hostname.endsWith(`.${domain}`),
    )
  );
}

export function hostname(input: string) {
  const host = input.trim();
  try {
    return new URL(/^https?:\/\//i.test(host) ? host : `https://${host}`)
      .hostname;
  } catch {
    return host;
  }
}

export class UntrustedLink extends Error {
  constructor(url: string) {
    super(`The sign-in link went to an unexpected address: ${url}`);
  }
}
