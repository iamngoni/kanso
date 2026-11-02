/**
 * Where to get Kanso Pocket, the iPhone, iPad and Android app. Set
 * `ANDROID_URL` to the Google Play listing when it is approved and every
 * "Coming soon" state on the site turns into a link.
 */

export const APP_STORE_ID = "6828301844";
export const APP_STORE_URL = `https://apps.apple.com/app/kanso-pocket/id${APP_STORE_ID}`;

/** The Google Play listing; null while the app is in review. */
export const ANDROID_URL: string | null = null;
