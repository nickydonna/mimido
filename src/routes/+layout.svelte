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
  <div class="mt-6 mb-16">
    {@render children()}
  </div>
</main>
<Navigation {activeUrl} defaultCalendar={data.defaultCalendar} />
