<script lang="ts">
  import { setContext } from "svelte";
  import NavigationButton from "./NavigationButton.svelte";
  // @ts-expect-error iconify
  import CalendarDateIcon from "~icons/solar/calendar-date-linear";
  // @ts-expect-error iconify
  import SettingsIcon from "~icons/solar/settings-broken";
  import type { Calendar } from "../../../bindings";
  import EventCreationModal from "../event-creation-modal/EventCreationModal.svelte";

  type Props = {
    activeUrl: string;
    disabled?: boolean;
    defaultCalendar: Calendar;
  };

  let props = $props();
  let { disabled = false, activeUrl, defaultCalendar }: Props = $derived(props);
  setContext<() => { disabled: boolean }>("navigation", () => ({
    disabled,
  }));
</script>

<div class="fixed z-[90] bottom-8 w-dvw">
  <nav class="container mx-auto flex justify-center items-center px-2 gap-2">
    <div
      class={[
        "navigation",
        "inline-flex rounded-full glassy-shadow",
        "p-1 gap-2",
      ]}
      role="group"
    >
      <NavigationButton href="/day" active={activeUrl === "/day"}>
        {#snippet icon(className: string)}
          <CalendarDateIcon class={className} />
        {/snippet}
        {#snippet label()}
          Day
        {/snippet}
      </NavigationButton>
      <NavigationButton href="/servers" active={activeUrl === "/servers"}>
        {#snippet icon(className: string)}
          <SettingsIcon class={className} />
        {/snippet}
        {#snippet label()}
          Setting
        {/snippet}
      </NavigationButton>
    </div>

    <EventCreationModal {defaultCalendar} />
  </nav>
</div>

<style lang="postcss">
  @reference "../../../app";

  .navigation {
    backdrop-filter: blur(1.5px);
  }
</style>
