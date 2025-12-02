<script lang="ts">
	import { Editor, createEditorContext } from '$lib/components/editor';
	import { MenuBar, FooterBar, Sidebar, Aside } from '$lib/components/layout';

	// Create editor context at page level
	const editorState = createEditorContext();
</script>

<MenuBar />
<Sidebar />
<main data-layout="main">
	<Editor />
</main>
<Aside />
<FooterBar />
