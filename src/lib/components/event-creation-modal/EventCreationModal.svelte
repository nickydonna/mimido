<script lang="ts">
  import { format, formatISO, isSameDay, parseISO } from "date-fns";

  import { commands, type DisplayUpsertInfo } from "../../../bindings";
  import { unwrap } from "$lib/result";
  // @ts-expect-error iconify
  import CalendarIcon from "~icons/uit/calendar";
  // @ts-expect-error iconify
  import SubjectIcon from "~icons/uit/subject";
  // @ts-expect-error iconify
  import ProcessIcon from "~icons/uit/process";
  // @ts-expect-error iconify
  import CompressIcon from "~icons/uit/compress";

  import HoverableIcon from "../hoverable-icon/HoverableIcon.svelte";
  import GlassButton from "../glass-button/GlassButton.svelte";
  import GlassInput from "../glass-input/GlassInput.svelte";
  import { timeStore } from "../../../stores/times";

  const { open } = $props<{ open: boolean; date: Date }>();
  const date = $timeStore;
  console.log(formatISO(date));
  let input = $state("@block Work today at 10-13 every weekday");
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
    console.log(unwrap(res));
    result = unwrap(res);
  }, 100);

  async function save() {
    await commands.saveEvent(6, formatISO(date), input);
  }

  $effect(() => {
    callParse(input);
  });
</script>

{#if !open}
  <div class="fixed w-dvw h-dvh inset-0 z-[100]">
    <div
      class="relative -mt-32 top-1/2 max-w-sm md:max-w-md lg:max-w mx-auto text-white glass-modal"
    >
      <GlassInput
        class="w-full outline-none text-white"
        bind:value={input}
        placeholder="Type your event information ..."
        oninput={(e) => callParse(e.currentTarget.value)}
      />

      <hr class="my-6 border-primary-100/50 -mx-6" />
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
          {#if result.starts_at != null && result.ends_at != null}
            <div class="glass-prop flex gap-1 h-9 px-3.5 py-2 text-sm">
              <HoverableIcon
                iconCmp={CalendarIcon}
                text="Date:"
                class="mt-0.5"
              />
              {format(parseISO(result.starts_at), "MMM dd 'at' HH:mm")}
              {#if isSameDay(result.starts_at, result.ends_at)}
                {format(parseISO(result.ends_at), "'until' HH:mm")}
              {:else}
                {format(parseISO(result.ends_at), "MMM dd 'at' HH:mm")}
              {/if}
            </div>
            {#if result.recurrence}
              <div class="flex gap-0.5 glass-prop h-9 px-3.5 py-2 text-sm">
                <HoverableIcon iconCmp={ProcessIcon} text="Recurrence:" />
                {result.recurrence}
              </div>
            {/if}
          {/if}
        </div>
        <hr class="my-6 border-primary-100/50 -mx-6" />
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
          <GlassButton onclick={save}>Save</GlassButton>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style lang="postcss">
  @reference "../../../app.css";
  .glass-modal {
    @apply glassy-shadow rounded-4xl p-6 bg-primary-800/30;
    backdrop-filter: blur(4px);
  }

  .glass-prop {
    @apply rounded-3xl bg-primary-900/50;
    text-wrap: nowrap;
  }
</style>
