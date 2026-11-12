/**
 * What one sync account can hold. These mirror the server's defaults in
 * docs/ACCOUNTS.md ("Quotas and capacity": `quota_doc_bytes`,
 * `quota_file_bytes`, `upload_bytes_per_day`) and cloud/migrations/0002_accounts.sql,
 * so a quota change must change them too (quota.test.ts pins the numbers).
 */
export const MIB = 1024 * 1024;
export const GIB = 1024 * MIB;

export const QUOTA_NOTES_BYTES = 100 * MIB;
export const QUOTA_FILES_BYTES = GIB;
export const QUOTA_UPLOAD_PER_DAY_BYTES = GIB;

/** "100 MiB", "1 GiB": whole units only, as the page prints them. */
export function formatSize(bytes: number): string {
  return bytes >= GIB && bytes % GIB === 0 ? `${bytes / GIB} GiB` : `${bytes / MIB} MiB`;
}
