import { error, redirect } from '@sveltejs/kit';
import { drafts } from '$lib/api';

export async function load({ params, parent }) {
  if (!params.id) {
    const { drafts: list } = await parent();
    if (list.length > 0) {
      redirect(307, `/drafts/${list[0].id}`);
    }
    return { draft: null };
  }

  if (params.id === 'new') {
    return { draft: null };
  }

  const id = Number(params.id);

  if (isNaN(id)) {
    error(400, 'Invalid draft ID');
  }

  const draft = await drafts.get(id);

  if (!draft) {
    error(404, 'Draft not found');
  }

  return { draft };
}
