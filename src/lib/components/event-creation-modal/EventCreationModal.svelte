<script lang="ts">
  import { format, formatISO, isSameDay, parseISO } from "date-fns";

  import { commands, type DisplayUpsertInfo } from "../../../bindings";
  import { unwrap } from "$lib/result";
  import { CalendarMonthOutline, ColumnOutline } from "flowbite-svelte-icons";
  import HoverableIcon from "../hoverable-icon/HoverableIcon.svelte";

  const { open, date } = $props<{ open: boolean; date: Date }>();

  let input = $state("@block hello tomorrow at 15:30");
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
  let typeClass = $derived(
    result != null ? `type-${result.event_type.toLowerCase()}` : "",
  );
</script>

{#if open}
  <div class="fixed w-dvw h-dvh inset-0 z-[100]">
    <div
      class="relative -mt-32 top-1/2 max-w-sm md:max-w-md lg:max-w mx-auto h-65 p-6 text-white"
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
        <div
          class="flex gap-0.5 my-4 glass-section h-12 px-4 py-3 rounded-3xl w-3/4"
        >
          <HoverableIcon
            iconCmp={CalendarMonthOutline}
            text="Summary:"
            class="mt-0.5"
          />
          {result.summary}
        </div>
        <div class="flex gap-2">
          <div
            class={[
              "flex gap-0.5 glass-section h-9 px-3.5 py-2 rounded-3xl text-sm",
              typeClass,
            ]}
          >
            <HoverableIcon iconCmp={ColumnOutline} text="Type:" />
            {result.event_type}
          </div>
          {#if result.starts_at != null && result.ends_at != null}
            <div class="glass-section h-9 px-3.5 py-2 rounded-3xl text-sm">
              {format(parseISO(result.starts_at), "MMM dd 'at' HH:mm")}
              {#if isSameDay(result.starts_at, result.ends_at)}
                {format(parseISO(result.ends_at), "'until' HH:mm")}
              {:else}
                {format(parseISO(result.ends_at), "MMM dd 'at' HH:mm")}
              {/if}
            </div>
          {/if}
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

  /* inspired in  https://atlaspuplabs.com/blog/liquid-glass-but-in-css?utm_source=tldrwebdev */
  @layer components {
    .glass-section {
      box-shadow:
        inset 10px 10px 20px rgba(153, 192, 255, 0.1),
        inset 2px 2px 5px rgba(195, 218, 255, 0.2),
        inset -10px -10px 20px rgba(229, 253, 190, 0.1),
        inset -2px -2px 30px rgba(247, 255, 226, 0.2);
      backdrop-filter: blur(20px);
    }
  }

  .type-block {
    @apply text-emerald-600;
  }
  .type-event {
    @apply text-indigo-600;
  }
  .type-task {
    @apply text-pink-600;
  }
  .type-reminder {
    @apply text-blue-600;
  }
</style>
