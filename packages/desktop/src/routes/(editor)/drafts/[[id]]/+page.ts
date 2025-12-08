import { error, redirect } from '@sveltejs/kit';
import { getDraft } from '$lib/api';

export async function load({ params, parent }) {
	// No id - redirect to first draft or stay for new draft mode
	if (!params.id) {
		const { drafts } = await parent();
		if (drafts.length > 0) {
			// Redirect to most recent draft
			redirect(307, `/drafts/${drafts[0].id}`);
		}
		// No drafts - new draft mode
		return { draft: null };
	}

	// "new" means new draft
	if (params.id === 'new') {
		return { draft: null };
	}

	const id = Number(params.id);

	if (isNaN(id)) {
		error(400, 'Invalid draft ID');
	}

	const draft = await getDraft(id);

	if (!draft) {
		error(404, 'Draft not found');
	}

	return { draft };
}
