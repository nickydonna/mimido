<script lang="ts">
  import "../app.css";

  import { CalendarEditOutline, UserOutline } from "flowbite-svelte-icons";
  import BottomNav from "flowbite-svelte/BottomNav.svelte";
  import BottomNavItem from "flowbite-svelte/BottomNavItem.svelte";

  import { sineIn } from "svelte/easing";
  import { Drawer } from "flowbite-svelte";
  import { page } from "$app/state";

  let { children } = $props();

  const transitionParams = {
    y: 320,
    duration: 200,
    easing: sineIn,
  };

  let activeUrl = $derived(page.url.pathname);
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  <div class="mt-6 mb-16">
    {@render children()}
  </div>
</main>
<Drawer
  width="w-full"
  transitionType="fly"
  placement="bottom"
  {transitionParams}
>
  Drawer
</Drawer>
<BottomNav
  position="fixed"
  classInner="grid-cols-4"
  navType="application"
  {activeUrl}
>
  <BottomNavItem btnName="Day" href="/" appBtnPosition="left">
    <CalendarEditOutline />
  </BottomNavItem>
  <BottomNavItem btnName="Server" href="/servers" appBtnPosition="right">
    <UserOutline />
  </BottomNavItem>
</BottomNav>
