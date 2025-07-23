<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import type { LayoutProps } from "./$types";
  import Navigation from "$lib/components/navigation/Navigation.svelte";

  let { children, data }: LayoutProps = $props();

  let activeUrl = $derived(page.url.pathname);
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  {@render children()}
</main>
<Navigation {activeUrl} defaultCalendar={data.defaultCalendar} />
