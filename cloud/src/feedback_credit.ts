// The credit a feedback report may ask for (docs/FEEDBACK.md › Credit): a plain name, or a GitHub login the
// filed issue @-mentions so GitHub notifies that person. A leaf module with no imports, so the intake
// (feedback.ts), the issue footer (feedback_triage.ts) and the GitHub search (github.ts) all share one rule.
//
// The two kinds are different promises. A name is the reporter's own words and can never become an
// @-mention: it can't contain `@` at all, and `creditText` strips one anyway. A login is written as
// `@login` only after it passes GitHub's own login rule here, so nothing but a real-looking login ever
// reaches the footer's mention position.

/** A name: letters, numbers, space, `.`, `_`, `-`; 1–39 characters. No `@`, so no mention. */
export const CREDIT_NAME = /^[\p{L}\p{N} ._-]{1,39}$/u;

/**
 * GitHub's login rule: 1–39 characters, ASCII letters, digits and single hyphens, no leading or
 * trailing hyphen (no two hyphens in a row). Enterprise-managed `_shortcode` logins are not matched.
 */
export const GITHUB_LOGIN = /^[A-Za-z0-9](?:[A-Za-z0-9]|-(?=[A-Za-z0-9])){0,38}$/;

/** The only host GitHub serves avatars from; a result with another avatar URL is dropped. */
export const AVATAR_URL = /^https:\/\/avatars\.githubusercontent\.com\/[A-Za-z0-9/_.?=&%-]{1,200}$/;

export type Credit = { kind: "name"; name: string } | { kind: "github"; login: string };

export type CreditChecked = { ok: true; value: Credit | null } | { ok: false; field: string };

const fail = (field: string): { ok: false; field: string } => ({ ok: false, field });
const isObject = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);

function nameOf(input: unknown, field: string): CreditChecked {
	if (typeof input !== "string") return fail(field);
	const name = input.normalize("NFC");
	return CREDIT_NAME.test(name) && name.trim() !== "" ? { ok: true, value: { kind: "name", name } } : fail(field);
}

/**
 * The credit of a report body, in its one stored form. Accepted: `null`; the tagged
 * `{kind: "name", name}` and `{kind: "github", login}`; and, for older app versions, the untagged
 * `{name}` and a bare string, both a plain name. Anything else fails at the path of the first bad part.
 */
export function parseCredit(input: unknown): CreditChecked {
	if (input === null) return { ok: true, value: null };
	if (typeof input === "string") return nameOf(input, "credit");
	if (!isObject(input)) return fail("credit");
	const keys = Object.keys(input).sort().join(",");
	if (keys === "name") return nameOf(input.name, "credit.name");
	if (keys === "kind,name") return input.kind === "name" ? nameOf(input.name, "credit.name") : fail("credit.kind");
	if (keys === "kind,login") {
		if (input.kind !== "github") return fail("credit.kind");
		return typeof input.login === "string" && GITHUB_LOGIN.test(input.login) ? { ok: true, value: { kind: "github", login: input.login } } : fail("credit.login");
	}
	return fail("credit");
}

/**
 * What the issue footer says after "Reported by ", and what `feedback.credit_name` keeps (so deleting
 * the report can find the line again): a login as `@login`, a name as itself. Null when there is
 * nothing safe to write. A name loses every `@`, as a second wall behind the pattern above.
 */
export function creditText(credit: Credit): string | null {
	if (credit.kind === "github") return GITHUB_LOGIN.test(credit.login) ? `@${credit.login}` : null;
	const name = credit.name.replaceAll("@", "").trim();
	return name === "" ? null : name;
}
