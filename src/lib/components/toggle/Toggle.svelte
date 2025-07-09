<script lang="ts">
  import { createSwitch } from "svelte-headlessui";

  let {
    label,
    checked = $bindable(),
    disabled = false,
    onchange,
  }: {
    label: string;
    checked: boolean;
    disabled?: boolean;
    onchange?: (value: boolean) => void;
  } = $props();

  const sw = createSwitch({ label, checked });

  // This seems too complex
  $effect(() => {
    if ($sw.checked !== checked && !disabled) {
      $sw.checked = checked;
    }
  });

  sw.subscribe((value) => {
    if (value.checked !== checked && !disabled) {
      onchange?.(value.checked);
    }
  });

  let buttonClass = $derived([
    $sw.checked ? "bg-teal-600/90" : "bg-teal-900/70",
    "relative inline-flex items-center h-8 w-18 shrink-0 cursor-pointer",
    "rounded-full border-3 border-transparent",
    "transition-colors duration-200 ease-in-out",
    "focus:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-200/75",
  ]);

  let pillClass = $derived([
    $sw.checked ? "translate-x-6.5" : "translate-x-0",
    "pointer-events-none inline-block h-6.5 w-10",
    "transform rounded-full bg-white ring-0 shadow-lg transition duration-200 ease-in-out",
  ]);
</script>

<button class={buttonClass} use:sw.toggle {disabled}>
  <span class="sr-only">{label}</span>
  <div class="absolute">
    {#if $sw.checked}
      <div class="text-white ml-3">|</div>
    {:else}
      <div class="text-teal-800 ml-12">0</div>
    {/if}
  </div>
  <span aria-hidden="true" class={pillClass}></span>
</button>
