import { listDrafts } from '$lib/api';

export async function load() {
  const drafts = await listDrafts();
  return { drafts };
}
