<script lang="ts">
  import "../app.css";
  // @ts-expect-error iconify
  import CalendarDateIcon from "~icons/solar/calendar-date-linear";
  // @ts-expect-error iconify
  import CalendarAddIcon from "~icons/solar/calendar-add-linear";
  // @ts-expect-error iconify
  import SettingsIcon from "~icons/solar/settings-broken";

  import { page } from "$app/state";
  import GlassIcon from "$lib/components/glass-icon/GlassIcon.svelte";
  import EventCreationModal from "$lib/components/event-creation-modal/EventCreationModal.svelte";
  import GlassButtonGroup from "$lib/components/glass-button-group/GlassButtonGroup.svelte";
  import GlassGrouppedButton from "$lib/components/glass-button-group/GlassGrouppedButton.svelte";

  let { children } = $props();

  let activeUrl = $derived(page.url.pathname);
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  <div class="mt-6 mb-16">
    {@render children()}
  </div>
</main>
<div class="fixed z-[90] bottom-8 w-dvw">
  <div class="container mx-auto flex items-center px-2 gap-2">
    <div class="flex-1"></div>
    <div>
      <GlassButtonGroup size="lg">
        <GlassGrouppedButton href="/day" active={activeUrl === "/day"}>
          <CalendarDateIcon />
        </GlassGrouppedButton>
        <GlassGrouppedButton href="/servers" active={activeUrl === "/servers"}>
          <SettingsIcon />
        </GlassGrouppedButton>
      </GlassButtonGroup>
    </div>
    <EventCreationModal />
  </div>
</div>
