<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    value = $bindable<string>(),
    ref = $bindable<HTMLInputElement | null>(),
    ...props
  }: HTMLInputAttributes & { ref: HTMLInputElement | null } = $props();
</script>

<div class="glass-input h-12 px-6 py-3 rounded-3xl shadow-emerald-200 shadow">
  <input
    bind:this={ref}
    class="w-full outline-none text-white"
    bind:value
    placeholder="Type your event information ..."
    {...props}
  />
</div>

<style lang="postcss">
  @reference "../../../app.css";

  input::placeholder {
    color: var(--color-neutral-300);
  }

  .glass-input {
    @apply glassy-shadow-white relative;
    transition: box-shadow 1s ease-in-out;
  }

  .glass-input::after {
    content: "";
    opacity: 0;
    transition: opacity 0.5s ease-in-out;
    pointer-events: none;
    border-radius: inherit; /* Ensure the shadow follows the button's border-radius */
    @apply absolute inset-0 w-full h-full shadow-primary-300 shadow;
  }

  .glass-input:has(input:focus)::after {
    opacity: 1;
  }
</style>
