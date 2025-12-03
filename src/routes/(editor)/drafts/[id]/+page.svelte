<script lang="ts">
	import { Editor } from '$lib/components/editor';
import { DraftState, setDraftState } from '$lib/stores/drafts.svelte';
	import { beforeNavigate } from '$app/navigation';

	let { data } = $props();

	const state = new DraftState(data.draft);
	setDraftState(state);

	beforeNavigate(() => state.flushPendingSave());
</script>

<Editor />
