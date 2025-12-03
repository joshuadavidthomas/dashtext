import { redirect } from '@sveltejs/kit';

export async function load({ parent }) {
	const { drafts } = await parent();

	if (drafts.length === 0) {
		redirect(307, '/drafts/new');
	}

	// Most recent is first (sorted by modified_at DESC)
	redirect(307, `/drafts/${drafts[0].id}`);
}
