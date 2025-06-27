<script lang="ts">
  import { format, formatISO, isSameDay, parseISO } from "date-fns";

  import { commands, type DisplayUpsertInfo } from "../../../bindings";
  import { unwrap } from "$lib/result";
  import CalendarIcon from "~icons/uit/calendar";
  import ProcessIcon from "~icons/uit/process";
  import CompressIcon from "~icons/uit/compress";
  import HoverableIcon from "../hoverable-icon/HoverableIcon.svelte";
  import GlassButton from "../glass-button/GlassButton.svelte";

  const { open, date } = $props<{ open: boolean; date: Date }>();

  let input = $state("@block hello tomorrow at 15:30 every Mon");
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

  $effect(() => {
    callParse(input);
  });
</script>

{#if open}
  <div class="fixed w-dvw h-dvh inset-0 z-[100]">
    <div
      class="relative -mt-32 top-1/2 max-w-sm md:max-w-md lg:max-w mx-auto text-white glass-modal"
    >
      <div class="glass-section h-12 px-6 py-3 rounded-3xl">
        <input
          class="w-full outline-none text-white"
          bind:value={input}
          placeholder="Type your event information ..."
          oninput={(e) => callParse(e.currentTarget.value)}
        />
      </div>
      {#if result != null}
        <div class="flex gap-0.5 my-4 glass-prop h-12 px-4 py-3">
          <HoverableIcon
            iconCmp={CalendarIcon}
            text="Summary:"
            class="mt-0.5"
          />
          {result.summary}
        </div>
        <div class="flex flex-wrap gap-2">
          <div class={["flex gap-0.5 glass-prop h-9 px-3.5 py-2 text-sm"]}>
            <HoverableIcon iconCmp={CompressIcon} text="Type:" />
            {result.event_type}
          </div>
          {#if result.starts_at != null && result.ends_at != null}
            <div class="glass-prop h-9 px-3.5 py-2 text-sm">
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
          <div class="flex-1">Press Enter to save ...</div>
          <GlassButton>Save</GlassButton>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style lang="postcss">
  @reference "../../../app.css";
  input::placeholder {
    color: var(--color-neutral-300);
  }

  .glass-modal {
    @apply rounded-4xl p-6 bg-primary-800/30;
    backdrop-filter: blur(4px);
  }

  /* inspired in  https://atlaspuplabs.com/blog/liquid-glass-but-in-css?utm_source=tldrwebdev */
  .glass-section {
    text-wrap: nowrap;
    @apply glassy-shadow;
    backdrop-filter: blur(20px);
  }
  .glass-prop {
    @apply rounded-3xl bg-primary-900/50;
    text-wrap: nowrap;
  }
</style>
