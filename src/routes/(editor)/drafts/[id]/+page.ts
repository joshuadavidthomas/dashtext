import { error } from '@sveltejs/kit';
import { getDraft } from '$lib/api';

export async function load({ params }) {
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
