<script lang="ts">
  import "../app.css";

  import { CalendarEditOutline } from "flowbite-svelte-icons";
  import BottomNav from "flowbite-svelte/BottomNav.svelte";
  import BottomNavItem from "flowbite-svelte/BottomNavItem.svelte";

  import { sineIn } from "svelte/easing";
  import { formatISO } from "date-fns/fp";
  import { Drawer } from "flowbite-svelte";
  import { page } from "$app/state";

  const transitionParams = {
    y: 320,
    duration: 200,
    easing: sineIn,
  };

  let date = page.url.searchParams.get("date") ?? formatISO(new Date());
</script>

<svelte:head></svelte:head>
<main class="container mx-auto h-full">
  <div class="mt-6 mb-16">
    <slot />
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
  activeUrl={page.url.pathname}
>
  <BottomNavItem btnName="Day" href="/day?date={date}" appBtnPosition="left">
    <CalendarEditOutline
      class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
    />
  </BottomNavItem>
</BottomNav>
