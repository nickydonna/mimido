<script lang="ts">
  import { createSwitch } from "svelte-headlessui";

  let {
    label,
    checked,
    onchange,
  }: { label: string; checked: boolean; onchange?: (value: boolean) => void } =
    $props();

  $effect(() => {
    console.log("c", checked);
    onchange?.(checked);
  });

  const sw = createSwitch({ label, checked: false });

  let classes = $derived([
    $sw.checked ? "translate-x-9" : "translate-x-0",
    "pointer-events-none inline-block h-[34px] w-[34px]",
    "transform rounded-full bg-white ring-0 shadow-lg transition duration-200 ease-in-out",
  ]);
</script>

<button
  class="relative inline-flex h-[38px] w-[74px] shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-teal-700 transition-colors duration-200 ease-in-out focus:outline-hidden focus-visible:ring-2 focus-visible:ring-white/75"
  use:sw.toggle
>
  <span class="sr-only">{label}</span>
  <span aria-hidden="true" class={classes}></span>
</button>
