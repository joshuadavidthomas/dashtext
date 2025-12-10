<script lang="ts">
  import '../app.css';
  import { setPlatformContext } from '@dashtext/lib/platform';
  import { webPlatform } from '$lib/platform';

  let { children } = $props();

  setPlatformContext(webPlatform);
</script>

{@render children()}
