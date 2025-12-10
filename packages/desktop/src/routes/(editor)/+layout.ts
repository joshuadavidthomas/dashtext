import { drafts } from '$lib/api';

export async function load() {
	const list = await drafts.list();
	return { drafts: list };
}
