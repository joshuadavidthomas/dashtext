<script lang="ts">
	import '../app.css';
	import { setPlatformContext } from '@dashtext/lib/platform';
	import { createSettingsContext, OnboardingDialog } from '@dashtext/lib';
	import { desktopPlatform } from '$lib/platform';

	let { children } = $props();

	setPlatformContext(desktopPlatform);
	createSettingsContext();
</script>

<OnboardingDialog />
{@render children()}
