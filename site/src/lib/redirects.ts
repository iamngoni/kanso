/**
 * The site's redirects, answered before TanStack Start sees the request:
 * www.kansoapp.app to the apex (permanent), and /metrics, the link the apps'
 * usage-count notice opens, to the note that explains it (temporary, so the
 * target can move without browsers remembering the old one).
 */
import { REPOSITORY_URL } from "./release";

export const APEX = "kansoapp.app";

/**
 * The published Kanso note of docs/METRICS-NOTE.md ("Kanso counts, it doesn't
 * watch"): what the apps' anonymous usage counts send and don't. Until that
 * note is published it points at the file on GitHub.
 */
export const METRICS_NOTE_URL = `${REPOSITORY_URL}/blob/master/docs/METRICS-NOTE.md`;

/** The redirect `url` gets, or null to serve it. */
export function redirectFor(url: URL): Response | null {
  if (url.hostname === `www.${APEX}`) {
    const apex = new URL(url);
    apex.hostname = APEX;
    return Response.redirect(apex.toString(), 301);
  }
  if (url.pathname === "/metrics" || url.pathname === "/metrics/") return Response.redirect(METRICS_NOTE_URL, 302);
  return null;
}
