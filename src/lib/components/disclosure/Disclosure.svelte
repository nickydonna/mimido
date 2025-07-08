<script lang="ts">
  // @ts-expect-error iconify
  import AngleUpIcon from "~icons/uit/angle-up";

  import { createDisclosure } from "svelte-headlessui";
  import type { Snippet } from "svelte";

  let {
    label,
    header,
    content,
    expanded = false,
  }: {
    label: string;
    expanded?: boolean;
    header: Snippet;
    content: Snippet;
  } = $props();

  const disclosure = createDisclosure({ label, expanded });
</script>

<div class="mb-2">
  <button use:disclosure.button class="disclosure-button">
    <span>{@render header()}</span>
    <AngleUpIcon
      class="h-8 w-8 self-center text-primary-100 {$disclosure.expanded
        ? ''
        : 'rotate-180 transform'}"
    />
  </button>
  {#if $disclosure.expanded}
    <div use:disclosure.panel class="px-4 pt-4 pb-2 text-sm text-gray-500">
      {@render content()}
    </div>
  {/if}
</div>

<style lang="postcss">
  @reference "../../../app";
  .disclosure-button {
    @apply flex w-full justify-between px-4 py-2;
    @apply rounded-lg bg-primary-600;
    @apply text-left text-sm font-medium text-primary-100;
    @apply hover:bg-primary-600 focus:outline-hidden focus-visible:ring-3 focus-visible:ring-primary-500/75;
  }
</style>
