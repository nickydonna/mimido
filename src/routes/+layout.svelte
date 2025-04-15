<script lang="ts">
  import "../app.css";

  import { CalendarEditOutline, UserOutline } from "flowbite-svelte-icons";
  import BottomNav from "flowbite-svelte/BottomNav.svelte";
  import BottomNavItem from "flowbite-svelte/BottomNavItem.svelte";

  import { page } from "$app/state";

  let { children } = $props();

  let activeUrl = $derived(page.url.pathname);
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  <div class="mt-6 mb-16">
    {@render children()}
  </div>
</main>
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
