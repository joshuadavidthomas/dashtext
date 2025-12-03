<script lang="ts">
	import { Editor } from '$lib/components/editor';
import { DraftState, setDraftState } from '$lib/stores/drafts.svelte';
	import { beforeNavigate } from '$app/navigation';

	const state = new DraftState(null);
	setDraftState(state);

	beforeNavigate(() => state.flushPendingSave());
</script>

<Editor />
