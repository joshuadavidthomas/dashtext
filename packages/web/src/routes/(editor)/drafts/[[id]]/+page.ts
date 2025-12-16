import { error, redirect } from '@sveltejs/kit';
import { drafts } from '$lib/api';
import type { DraftData } from '@dashtext/lib';
import type { PageLoadEvent } from './$types';

export async function load({ params, parent }: PageLoadEvent) {
	// No id - redirect to first draft or stay for new draft mode
	if (!params.id) {
		const { drafts: list } = await parent();
		if (list.length > 0) {
			// Redirect to most recent draft
			redirect(307, `/drafts/${list[0].uuid}`);
		}
		// No drafts - new draft mode
		return { draft: null };
	}

	// "new" means new draft
	if (params.id === 'new') {
		return { draft: null };
	}

	// Fetch draft by UUID
	const draft = await drafts.get(params.id);
	
	if (!draft) {
		error(404, 'Draft not found');
	}

	return { draft };
}
