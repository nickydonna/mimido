<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  let { value = $bindable<string>(), ...props }: HTMLInputAttributes = $props();
</script>

<div class="glass-input h-12 px-6 py-3 rounded-3xl shadow-emerald-200 shadow">
  <input
    class="w-full outline-none text-white"
    bind:value
    placeholder="Type your event information ..."
    {...props}
  />
</div>

<style lang="postcss">
  @reference "../../../app.css";

  .glass-input {
    @apply glassy-shadow relative;
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
