<script lang="ts">
  import "../app.css";
  import { CalendarEditOutline, UserOutline } from "flowbite-svelte-icons";
  import BottomNav from "flowbite-svelte/BottomNav.svelte";
  import BottomNavItem from "flowbite-svelte/BottomNavItem.svelte";
  // @ts-expect-error iconify
  import CalendarAddIcon from "~icons/solar/calendar-add-linear";

  import { page } from "$app/state";
  import GlassIcon from "$lib/components/glass-icon/GlassIcon.svelte";
  import EventCreationModal from "$lib/components/event-creation-modal/EventCreationModal.svelte";

  let { children } = $props();
  let open = $state(false);

  let activeUrl = $derived(page.url.pathname);
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  <div class="mt-6 mb-16">
    {@render children()}
  </div>
</main>
<GlassIcon class="fixed! bottom-6 right-6 z-[90]" onclick={() => (open = true)}>
  <CalendarAddIcon />
</GlassIcon>
<EventCreationModal {open} onclose={() => (open = false)} />
<BottomNav
  position="sticky"
  navType="application"
  {activeUrl}
  outerClass="z-50 "
  innerClass="grid-cols-2 bg-primary-500 rounded-full border-primary-700"
>
  <BottomNavItem btnName="Day" href="/day" appBtnPosition="left">
    <CalendarEditOutline />
  </BottomNavItem>
  <BottomNavItem btnName="Server" href="/servers" appBtnPosition="right">
    <UserOutline />
  </BottomNavItem>
</BottomNav>
