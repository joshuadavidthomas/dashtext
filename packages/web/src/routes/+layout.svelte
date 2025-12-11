<script lang="ts">
  import '../app.css';
  import { setPlatformContext } from '@dashtext/lib/platform';
  import { createSettingsContext, OnboardingDialog } from '@dashtext/lib';
  import { webPlatform } from '$lib/platform';

  let { children } = $props();

  setPlatformContext(webPlatform);
  createSettingsContext();
</script>

<OnboardingDialog />
{@render children()}
