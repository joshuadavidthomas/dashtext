<script lang="ts">
  import '../app.css';
  import { setPlatformContext } from '@dashtext/lib/platform';
  import { createSettingsContext, OnboardingDialog } from '@dashtext/lib';
  import { webPlatform } from '$lib/platform';
  // Import diagnostics to make them available in console
  import '$lib/api/db-diagnostics';

  let { children } = $props();

  setPlatformContext(webPlatform);
  createSettingsContext();
</script>

<OnboardingDialog />
{@render children()}
