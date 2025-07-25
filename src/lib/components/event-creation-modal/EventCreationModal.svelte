<script lang="ts">
  import {
    format,
    formatISO,
    getHours,
    getMinutes,
    isSameDay,
    isToday,
    isTomorrow,
    parseISO,
  } from "date-fns";
  import { createDialog } from "svelte-headlessui";
  import Transition from "svelte-transition";

  import {
    commands,
    type Calendar,
    type DisplayUpsertInfo,
  } from "../../../bindings";
  import { match, def } from "@korkje/adt";
  import { unwrap } from "$lib/result";
  // @ts-expect-error iconify
  import ClockIcon from "~icons/solar/clock-circle-broken";
  // @ts-expect-error iconify
  import SubjectIcon from "~icons/solar/text-field-broken";
  // @ts-expect-error iconify
  import HistoryBoldIcon from "~icons/solar/history-bold";
  // @ts-expect-error iconify
  import CompressIcon from "~icons/solar/posts-carousel-horizontal-line-duotone";
  // @ts-expect-error iconify
  import MultiplyIcon from "~icons/uit/multiply";
  // @ts-expect-error iconify
  import RoutingIcon from "~icons/solar/routing-line-duotone";

  import HoverableIcon from "../hoverable-icon/HoverableIcon.svelte";
  import GlassButton from "../glass-button/GlassButton.svelte";
  import GlassInput from "../glass-input/GlassInput.svelte";
  import { timeState } from "../../../stores/times.svelte";
  import GlassIcon from "../glass-icon/GlassIcon.svelte";
  import {
    EventUpsert,
    eventUpserter,
    isUpdating,
  } from "../../../stores/eventUpserter.svelte";
  import { invalidateAll } from "$app/navigation";
  import type { EventHandler } from "svelte/elements";

  let { defaultCalendar }: { defaultCalendar: Calendar | undefined } = $props();

  const dialog = createDialog({ label: "Upsert Event" });
  const date = timeState.time;
  let input = $state("");
  let result = $state<DisplayUpsertInfo | null>(null);

  /**
   * Debounce function to limit the rate at which a function can fire.
   * @param func
   * @param wait
   */
  function debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number,
  ): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout> | null = null;
    return function executedFunction(...args: Parameters<T>) {
      const later = () => {
        timeout = null;
        func(...args);
      };
      if (timeout !== null) {
        clearTimeout(timeout);
      }
      timeout = setTimeout(later, wait);
    };
  }

  const callParse = debounce(async (input: string) => {
    if (input.length <= 3) {
      return;
    }
    const res = await commands.parseEvent(formatISO(date), input);
    result = unwrap(res);
  }, 100);

  function dateToString(time: Date): string {
    const minutes = getMinutes(time);
    if (isToday(time)) {
      return minutes === 0
        ? `today at ${getHours(time)}`
        : `today at ${getHours(time)}:${getMinutes(time)}`;
    }

    if (isTomorrow(time)) {
      return minutes === 0
        ? `tomorrow at ${getHours(time)}`
        : `tomorrow at ${getHours(time)}:${getMinutes(time)}`;
    }

    return minutes === 0
      ? `at ${format(time, "dd/MM h")}`
      : `at ${format(time, "dd/MM h:m")}`;
  }

  let ref = $state<HTMLInputElement | null>(null);

  $effect(() => {
    match(eventUpserter.state, {
      Creating: ({ type, startDate }) => {
        dialog.open();
        input = `.${type.toLowerCase()} ${dateToString(startDate)} `;
        // Set some delay to wait for things to render
        setTimeout(() => {
          ref?.focus();
        }, 10);
      },
      Updating: ({ event }) => {
        dialog.open();
        input = event.natural_string;
        // Set some delay to wait for things to render
        setTimeout(() => {
          ref?.focus();
        }, 10);
      },
      None: () => dialog.close(),
    });
  });

  let loading = $state(false);
  const save: EventHandler = async (e: Event) => {
    e.preventDefault();

    if (defaultCalendar == null) {
      alert("Please pick a default calendar");
      return;
    }
    loading = true;
    if (isUpdating(eventUpserter.state)) {
      await commands.updateVevent(
        eventUpserter.state[1].event.id,
        formatISO(date),
        input,
      );
    } else {
      await commands.saveEvent(defaultCalendar.id, formatISO(date), input);
    }

    await invalidateAll();
    eventUpserter.state = EventUpsert.None;
    loading = false;
  };

  const deleteEvent = async () => {
    if (!isUpdating(eventUpserter.state)) {
      return;
    }
    loading = true;
    const [_, data] = eventUpserter.state;
    await commands.deleteVevent(data.event.id);
    await invalidateAll();
    eventUpserter.state = EventUpsert.None;
    loading = false;
  };

  $effect(() => {
    callParse(input);
  });

  let actionStr = $derived(
    match(eventUpserter.state, {
      Updating: () => "Update",
      Creating: () => "Create",
      None: () => "",
    }),
  );

  function handleClose() {
    eventUpserter.state = EventUpsert.None;
  }
</script>

{#snippet hr()}
  <div class="my-4 h-0.5 bg-primary-100/30 -mx-5 rounded"></div>
{/snippet}

<Transition
  show={$dialog.expanded}
  enter="ease-in-out duration-300"
  enterFrom="opacity-0"
  enterTo="opacity-100"
  leave="ease-in-out duration-300"
  leaveFrom="opacity-100"
  leaveTo="opacity-0"
>
  <div class="fixed w-dvw h-dvh inset-0 z-[100] glass-modal-backdrop">
    <form
      onsubmit={save}
      use:dialog.modal
      onclose={handleClose}
      class="relative -mt-32 top-1/2 max-w-sm md:max-w-md lg:max-w mx-auto text-white glass-modal"
    >
      <div class="flex items-center gap-3 w-full mb-2">
        <div class="flex-1">
          {actionStr}
          {#if defaultCalendar != null}
            at
            <span class="text-lg text-primary-200 underline"
              >{defaultCalendar.name}</span
            >
          {/if}
        </div>

        <GlassIcon size="xs" onclick={handleClose}>
          <MultiplyIcon />
        </GlassIcon>
      </div>
      {@render hr()}
      <GlassInput
        bind:ref
        disabled={loading}
        class="w-full outline-none text-white"
        bind:value={input}
        placeholder="Type your event information ..."
      />
      {@render hr()}
      {#if result != null}
        <div class="flex gap-0.5 my-4 glass-prop h-12 px-4 py-3">
          <HoverableIcon iconCmp={SubjectIcon} text="Summary:" class="mt-0.5" />
          {result.summary}
        </div>
        <div class="flex flex-wrap gap-2">
          <div class={["flex gap-0.5 glass-prop h-9 px-3.5 py-2 text-sm"]}>
            <HoverableIcon iconCmp={CompressIcon} text="Type:" />
            {result.event_type}
          </div>
          <div class={["flex gap-0.5 glass-prop h-9 px-3.5 py-2 text-sm"]}>
            <HoverableIcon iconCmp={RoutingIcon} text="Status:" />
            {result.status}
          </div>

          {#if result.starts_at != null && result.ends_at != null}
            <div class="glass-prop flex gap-1 h-9 px-3.5 py-2 text-sm">
              <HoverableIcon iconCmp={ClockIcon} text="Date:" />
              {format(parseISO(result.starts_at), "MMM dd 'at' HH:mm")}
              {#if isSameDay(result.starts_at, result.ends_at)}
                {format(parseISO(result.ends_at), "'until' HH:mm")}
              {:else}
                {format(parseISO(result.ends_at), "MMM dd 'at' HH:mm")}
              {/if}
            </div>
            {#if result.recurrence}
              <div class="flex gap-0.5 glass-prop h-9 px-3.5 py-2 text-sm">
                <HoverableIcon iconCmp={HistoryBoldIcon} text="Recurrence:" />
                {result.recurrence}
              </div>
            {/if}
          {/if}
        </div>
        {@render hr()}
        <div class="flex gap-2 items-center">
          {#if isUpdating(eventUpserter.state)}
            <div class="flex-1"></div>
            <GlassButton size="xs" onclick={deleteEvent} disabled={loading}>
              Delete
            </GlassButton>
          {:else}
            <div class="flex-1">
              Press
              <span
                class="mx-1 p-1 rounded bg-emerald-700 shadow shadow-emerald-400"
              >
                ↵
              </span>
              to {actionStr.toLowerCase()} ...
            </div>
          {/if}
          <GlassButton type="submit" {loading}>{actionStr}</GlassButton>
        </div>
      {/if}
    </form>
  </div>
</Transition>

<style lang="postcss">
  @reference "../../../app.css";

  .glass-modal-backdrop {
    @apply bg-black/20;
    backdrop-filter: blur(0.5px);
  }

  .glass-modal {
    @apply glassy-shadow rounded-4xl p-6 bg-primary-800/30;
    backdrop-filter: blur(4px);
  }

  .glass-prop {
    @apply rounded-3xl bg-primary-900/50;
    text-wrap: nowrap;
  }
</style>
