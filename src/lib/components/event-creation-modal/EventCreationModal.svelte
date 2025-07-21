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
    eventUpsert,
    eventUpserter,
  } from "../../../stores/eventUpserter.svelte";

  let { defaultCalendar }: { defaultCalendar: Calendar | undefined } = $props();

  const dialog = createDialog({ label: "Create Event" });
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
      None: () => dialog.close(),
      Creating: ({ type, startDate }) => {
        dialog.open();
        input = `@${type.toLowerCase()} ${dateToString(startDate)} `;
        // Set some delay to wait for things to render
        setTimeout(() => {
          ref?.focus();
        }, 10);
      },
      [def]: () => dialog.open(),
    });
  });

  let saving = $state(false);
  async function save() {
    if (defaultCalendar == null) {
      alert("Please pick a default calendar");
      return;
    }
    saving = true;
    await commands.saveEvent(defaultCalendar.id, formatISO(date), input);
    saving = false;
  }

  $effect(() => {
    callParse(input);
  });
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
    <div
      use:dialog.modal
      onclose={() => {
        eventUpserter.state = eventUpsert.None;
      }}
      class="relative -mt-32 top-1/2 max-w-sm md:max-w-md lg:max-w mx-auto text-white glass-modal"
    >
      <div class="flex items-center gap-3 w-full mb-2">
        <div class="flex-1">
          Create {result?.event_type ?? "Event"}
          {#if defaultCalendar != null}
            at
            <span class="text-lg text-primary-200 underline"
              >{defaultCalendar.name}</span
            >
          {/if}
        </div>

        <GlassIcon
          size="xs"
          onclick={() => {
            eventUpserter.state = eventUpsert.None;
          }}
        >
          <MultiplyIcon />
        </GlassIcon>
      </div>
      {@render hr()}
      <GlassInput
        bind:ref
        disabled={saving}
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
        <div class="flex items-center">
          <div class="flex-1">
            Press
            <span
              class="mx-1 p-1 rounded bg-emerald-700 shadow shadow-emerald-400"
            >
              ↵
            </span>
            to save ...
          </div>
          <GlassButton loading={saving} onclick={save}>Save</GlassButton>
        </div>
      {/if}
    </div>
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
