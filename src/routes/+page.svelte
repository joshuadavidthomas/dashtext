<script lang="ts">
	import { Editor, createEditorContext } from '$lib/components/editor';
	import { MenuBar, FooterBar, Aside } from '$lib/components/layout';
	import AppSidebar from '$lib/components/app-sidebar.svelte';

	// Create editor context at page level
	const editorState = createEditorContext();
</script>

<MenuBar />
<AppSidebar />
<main data-layout="main">
	<Editor />
</main>
<Aside />
<FooterBar />
